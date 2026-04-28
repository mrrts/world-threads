# /eureka — worldcli rubric refs can carry criterion middleware

*Generated 2026-04-28 08:30. This run opened to test whether any `worldcli`
surface had crossed into the same criterion-setting role named in the prior
eureka, or whether that layer belonged mostly to corrective reports. The answer
is yes: the rubric library is already carrying it.*

## The discovery

`worldcli` does have a surface that carries criterion middleware:

- the **rubric library**, especially `evaluate --rubric-ref <name>`

Corrective reports still do the original naming and correction work.

But once that corrected boundary is embodied in a named rubric file and reused
through `--rubric-ref`, the tool surface itself begins transporting the
criterion into later experiments.

## Why this became visible

The project already had the ingredients in plain sight:

1. corrective reports that rewrote what the instrument was actually measuring
2. rubric files in `reports/rubrics/` that absorbed those corrections as formal
   boundaries, worked examples, and known failure modes
3. a `worldcli` command shape that reuses those rubric files by name

That means later evaluations do not start from scratch.

They inherit a previously-corrected boundary.

## Worked examples

### 1. Weight-carrier rubric

[reports/2026-04-23-1304-weight-carrier-refuted-but-interesting.md](/Users/ryansmith/Sites/rust/world-chat/reports/2026-04-23-1304-weight-carrier-refuted-but-interesting.md)
and
[reports/2026-04-23-1326-john-stillness-refuted-register-still-elsewhere.md](/Users/ryansmith/Sites/rust/world-chat/reports/2026-04-23-1326-john-stillness-refuted-register-still-elsewhere.md)
surfaced two criterion corrections:

- caution-adjacent language can be misread as reduction
- John's short pastoral register falls outside this rubric's YES boundary

Those corrections now live inside
[reports/rubrics/weight-carrier-hold-vs-reduce.md](/Users/ryansmith/Sites/rust/world-chat/reports/rubrics/weight-carrier-hold-vs-reduce.md)
under `Known failure modes`.

So when a later run uses:

- `worldcli evaluate ... --rubric-ref weight-carrier-hold-vs-reduce`

it is not merely reusing a scoring template.

It is inheriting the corrective understanding of what this rubric can and
cannot honestly claim.

### 2. Mission-adherence v3

[reports/rubrics/mission-adherence-v3.md](/Users/ryansmith/Sites/rust/world-chat/reports/rubrics/mission-adherence-v3.md)
is another strong case.

That rubric encodes corrections from prior drift:

- stretched tag citation does not count
- absent tags must not be invented
- warmly-competent is not the same as mission-advancing

Those are criterion corrections first, rubric text second.

Once encoded in the library, `worldcli evaluate --rubric-ref mission-adherence-v3`
is carrying forward a corrected interpretation frame, not just running a
generic evaluator.

## Why this matters

This sharpens the answer to the question "does criterion middleware live only in
reports?"

No.

The report is where the correction is usually discovered and argued.

But the moment that correction becomes a named reusable rubric, the worldcli
surface starts carrying it too.

That matters because it changes what kind of surface the rubric library is.

It is not only a convenience catalog.

It is a persistence layer for methodological correction.

## Scope

This should stay narrow.

Not every `worldcli` surface carries criterion middleware.

The strong case here is specifically:

- corrective finding
- embodied in rubric text
- reused by `--rubric-ref`

Other `worldcli` surfaces may still be evidence-only or braid-only unless they
inherit corrections in the same concrete way.

## Reading

The middleware family gets one more useful refinement:

- reports often discover criterion corrections
- rubric refs can operationalize those corrections

So the worldcli answer is not "the tool creates criterion middleware from
nothing."

It is:

the tool can become the delivery surface by which criterion middleware keeps
traveling after a corrective report has named it.
