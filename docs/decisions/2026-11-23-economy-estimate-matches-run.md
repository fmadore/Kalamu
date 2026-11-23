# 2026-11-23 · Economy mode is priced as the run batches, source by source

## Context

The repository audit of 23 November found that economy mode's estimate and its run disagreed
(#242). The estimate offered the batch price whenever the provider batches the task and model;
the run, deciding source by source, sends two kinds of work at once whatever the pace, because
their answers are checked against the source as they come in, which a batch's collection does
not do:

- a transcript translated line by line, each answer read back against the lines its passage
  sent (#167);
- an analysis whose fields cite the text, each quotation looked for in the text: research-question
  screening and interview analysis, what 1.9.0 and 1.10.0 brought. The estimate dropped the
  discount only for a text read in parts; a short text kept it.

A job mixing sources of both kinds (a timed transcript and plain letters, translated together)
had one answer per set-up in the estimate and several in the run. A researcher who chose the
slower mode to pay half paid the full price, and the figure shown before Start was wrong.

## Decision

- **One rule** decides whether a set-up's work on a source goes in a batch:
  `engine::batched::in_batch`, its provider batching the task with its model and the work not
  one of the two above (`batched::at_once`). The run partitions each source's pieces by it; the
  estimate prices by it. `aligned::timed_lines` reads the same test of timed lines.
- **The estimate applies it source by source and adds up.** Where none of a set-up's sources is
  batched, its row has no economy price, as a model without a batch service has none, and says
  why (`EstimateRow::at_once`: `quotes` or `timed_lines`). Where only some are, the discount is
  taken back on the others, priced alone.
- **The economy choice says so.** Under *Speed and price*, New job names each model whose work
  gets no discount, with the reason: no batch service, quotations checked in the text, or lines
  checked one by one. A step after the others (a table of contents' combination, the step that
  brings a long text's parts together) is never batched and is no longer named as a model
  without a batch service. When nothing in the job can be batched, *Cheaper* is not offered, as
  before for models without a batch service.

## Consequences

- An economy job of screening or interview analysis, or a timed translation, is estimated at
  the price it is billed; a mixed job at what its batched and unbatched pieces cost. Engine tests
  run each in economy mode against a provider with a batch service and compare the estimate with
  the run's routes priced from the catalogue.
- Changing what goes in a batch is one function; the estimate follows without being touched.
- Giving the discount back, by checking quotations and timed lines when a batch is collected,
  stays with #167.
