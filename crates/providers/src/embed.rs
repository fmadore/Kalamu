//! What every cloud embedder shares: a call checked against its limits before anything is sent,
//! the answer checked after (one vector for each text, all of one length) and every vector
//! brought to length 1, and the request shape OpenAI and Mistral have in common.
//!
//! Each provider counts its limit in tokens, which this side cannot count without the
//! provider's own tokenizer. English runs about four characters a token, Arabic two or three,
//! Chinese and Japanese about one, and a script a tokenizer barely knows can fall back to its
//! bytes, two or three tokens for one character. Kalamu assumes the worst it can expect, two
//! tokens a character ([`TOKENS_PER_CHAR`]), so that a text cut at [`EmbedLimits::chars`] never
//! passes the model's limit, whatever its script.

use kalamu_core::{ApiKey, EmbedLimits, EmbedRequest, Embedded, ProviderError};
use reqwest::Client;
use serde_json::{Value, json};

use crate::http::send_json;

/// The most tokens one character is assumed to make, in any script.
pub(crate) const TOKENS_PER_CHAR: usize = 2;

/// The characters one text may hold for a model that takes `tokens` tokens in one text, keeping
/// `reserved` of them for what the embedder adds (a prefix), rounded down to the hundred.
pub(crate) const fn chars_for(tokens: usize, reserved: usize) -> usize {
    (tokens - reserved) / TOKENS_PER_CHAR / 100 * 100
}

/// Whether there is anything to send: a call with no text answers at once with no vector, and
/// a call that breaks the limits, or comes after the job was stopped, is refused before it goes.
pub(crate) fn check(
    request: &EmbedRequest<'_>,
    limits: EmbedLimits,
) -> Result<Option<Embedded>, ProviderError> {
    if request.stop.is_stopped() {
        return Err(ProviderError::Interrupted);
    }
    if request.texts.is_empty() {
        return Ok(Some(Embedded {
            vectors: Vec::new(),
            input_tokens: 0,
        }));
    }
    if request.texts.len() > limits.texts {
        return Err(ProviderError::InvalidRequest(format!(
            "{} texts were sent in one call; {} takes at most {}",
            request.texts.len(),
            request.model,
            limits.texts
        )));
    }
    if request.dimensions == Some(0) {
        return Err(ProviderError::InvalidRequest(
            "vectors of length 0 were asked for".into(),
        ));
    }
    for (i, text) in request.texts.iter().enumerate() {
        if text.trim().is_empty() {
            return Err(ProviderError::InvalidRequest(format!(
                "text {} is empty: there is nothing to embed",
                i + 1
            )));
        }
        let chars = text.chars().count();
        if chars > limits.chars {
            return Err(ProviderError::InvalidRequest(format!(
                "text {} is too long for {}: {chars} characters, where it takes at most {} in one text",
                i + 1,
                request.model,
                limits.chars
            )));
        }
    }
    Ok(None)
}

/// A JSON array of numbers as a vector; `None` for anything else.
pub(crate) fn vector(value: Option<&Value>) -> Option<Vec<f32>> {
    value?
        .as_array()?
        .iter()
        .map(|n| n.as_f64().map(|n| n as f32))
        .collect()
}

/// The answer, checked: one vector for each text, all of the length asked for (or of one
/// length, when none was asked), each brought to length 1.
pub(crate) fn finish(
    request: &EmbedRequest<'_>,
    vectors: Vec<Vec<f32>>,
    input_tokens: u64,
) -> Result<Embedded, ProviderError> {
    if vectors.len() != request.texts.len() {
        return Err(ProviderError::Other(format!(
            "{} vectors came back for {} texts",
            vectors.len(),
            request.texts.len()
        )));
    }
    let expected = request
        .dimensions
        .map(|d| d as usize)
        .or_else(|| vectors.first().map(Vec::len))
        .unwrap_or(0);
    let vectors = vectors
        .into_iter()
        .enumerate()
        .map(|(i, v)| {
            if v.len() != expected || v.is_empty() {
                return Err(ProviderError::Other(format!(
                    "the vector for text {} has {} numbers where {expected} were expected",
                    i + 1,
                    v.len()
                )));
            }
            normalised(v).ok_or_else(|| {
                ProviderError::Other(format!(
                    "the vector for text {} is empty or not a number and cannot be compared",
                    i + 1
                ))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Embedded {
        vectors,
        input_tokens,
    })
}

/// A vector of length 1 pointing the same way; `None` for one of length 0 or holding a number
/// that is not finite. A vector already of length 1 comes back as it was, give or take rounding.
pub(crate) fn normalised(mut v: Vec<f32>) -> Option<Vec<f32>> {
    let norm = v
        .iter()
        .map(|x| f64::from(*x) * f64::from(*x))
        .sum::<f64>()
        .sqrt();
    if !norm.is_finite() || norm == 0.0 {
        return None;
    }
    for x in &mut v {
        *x = (f64::from(*x) / norm) as f32;
    }
    Some(v)
}

/// A call to `{base}/embeddings` in the shape OpenAI and Mistral share: the texts as `input`,
/// the length asked for under `dimensions_field`, and `data[].embedding` with its `index` and
/// `usage.prompt_tokens` back. The key goes in the request's header and nowhere else.
pub(crate) async fn embeddings_endpoint(
    client: &Client,
    base_url: &str,
    key: Option<&ApiKey>,
    request: &EmbedRequest<'_>,
    dimensions_field: &str,
) -> Result<Embedded, ProviderError> {
    let key = key.ok_or_else(|| ProviderError::Auth("no API key was given".into()))?;
    let mut body = json!({
        "model": request.model,
        "input": request.texts,
        "encoding_format": "float",
    });
    if let Some(dimensions) = request.dimensions {
        body[dimensions_field] = json!(dimensions);
    }
    let response = send_json(
        client
            .post(format!("{base_url}/embeddings"))
            .bearer_auth(key.expose())
            .json(&body),
    )
    .await?;
    let mut indexed = response
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| ProviderError::Other("no vectors in the response".into()))?
        .iter()
        .enumerate()
        .map(|(position, item)| {
            let index = item
                .get("index")
                .and_then(Value::as_u64)
                .map_or(position, |i| i as usize);
            let v = vector(item.get("embedding")).ok_or_else(|| {
                ProviderError::Other(format!("vector {} is not a list of numbers", position + 1))
            })?;
            Ok((index, v))
        })
        .collect::<Result<Vec<_>, ProviderError>>()?;
    // The answer lists each vector with the place of its text, in the order asked; one out of
    // place is put back, one missing or twice is refused.
    indexed.sort_by_key(|(i, _)| *i);
    if indexed.iter().enumerate().any(|(at, (i, _))| at != *i) {
        return Err(ProviderError::Other(
            "the vectors do not match the texts sent, one for each".into(),
        ));
    }
    let tokens = response
        .pointer("/usage/prompt_tokens")
        .or_else(|| response.pointer("/usage/total_tokens"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    finish(
        request,
        indexed.into_iter().map(|(_, v)| v).collect(),
        tokens,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use kalamu_core::{EmbedRole, StopSignal};

    fn request<'a>(texts: &'a [String], stop: &'a StopSignal) -> EmbedRequest<'a> {
        EmbedRequest {
            model: "m",
            role: EmbedRole::Passage,
            texts,
            dimensions: None,
            stop,
        }
    }

    const LIMITS: EmbedLimits = EmbedLimits {
        texts: 2,
        chars: 10,
    };

    #[test]
    fn vectors_are_brought_to_length_one() {
        let v = normalised(vec![3.0, 4.0]).unwrap_or_default();
        assert!((v[0] - 0.6).abs() < 1e-6 && (v[1] - 0.8).abs() < 1e-6);
        assert_eq!(normalised(vec![0.0, 0.0]), None);
        assert_eq!(normalised(vec![f32::NAN, 1.0]), None);
        assert_eq!(normalised(vec![f32::INFINITY, 1.0]), None);
    }

    #[test]
    fn a_call_outside_the_limits_is_refused_before_it_is_sent() {
        let stop = StopSignal::default();
        let ok = ["court".to_string()];
        assert_eq!(check(&request(&ok, &stop), LIMITS), Ok(None));
        let none: [String; 0] = [];
        assert!(matches!(
            check(&request(&none, &stop), LIMITS),
            Ok(Some(e)) if e.vectors.is_empty()
        ));
        let many = ["a".to_string(), "b".into(), "c".into()];
        assert!(matches!(
            check(&request(&many, &stop), LIMITS),
            Err(ProviderError::InvalidRequest(m)) if m.contains("at most 2")
        ));
        // Counted in characters, not bytes: ten Arabic letters fit.
        let arabic = ["مممممممممم".to_string()];
        assert_eq!(check(&request(&arabic, &stop), LIMITS), Ok(None));
        let long = ["onze lettres".to_string()];
        assert!(matches!(
            check(&request(&long, &stop), LIMITS),
            Err(ProviderError::InvalidRequest(m)) if m.contains("too long")
        ));
        let blank = ["  ".to_string()];
        assert!(check(&request(&blank, &stop), LIMITS).is_err());
        stop.stop();
        assert_eq!(
            check(&request(&ok, &stop), LIMITS),
            Err(ProviderError::Interrupted)
        );
    }

    #[test]
    fn the_limit_in_characters_leaves_room_for_two_tokens_each() {
        assert_eq!(chars_for(8_192, 0), 4_000);
        assert_eq!(chars_for(8_192, 64), 4_000);
        assert_eq!(chars_for(2_048, 0), 1_000);
    }
}
