# lora-validator Specification

## ADDED Requirements

### Requirement: LoRA validator replays failed prompts

The `nebula-lora-validator` Wasm component SHALL react to completed LoRA training events by replaying failed prompts against the newly published adapter through `tachyon:inference`.

#### Scenario: Training completion triggers replay

- **GIVEN** a `nebula.training.complete` event includes a LoRA artifact reference
- **AND** the failed prompt repository contains prompts from the related training batch
- **WHEN** the validator handles the event
- **THEN** it samples up to 20 failed prompts
- **AND** it generates one low-temperature and two high-temperature responses for each prompt using the adapter

### Requirement: LoRA validator asks the AST microVM to score replayed triplets

The validator SHALL submit each replayed response triplet to the `EvaluateTriplets` Protobuf contract served by the `nebula-eval-ast` MicroVM.

#### Scenario: Replay results are evaluated

- **GIVEN** replay generated three responses for a failed prompt
- **WHEN** the validator scores the replay
- **THEN** it sends an `EvaluationRequest` containing the language and responses to the AST evaluator
- **AND** it records whether the evaluator still reports divergence

### Requirement: LoRA validator emits validation decisions

The validator SHALL emit a validation success event only when every sampled prompt no longer diverges, and SHALL emit a failed event otherwise.

#### Scenario: Adapter passes validation

- **GIVEN** every replayed prompt receives an AST evaluation with `diverged == false`
- **WHEN** validation completes
- **THEN** the validator emits `nebula.validation.success`
- **AND** the payload includes the adapter artifact, pass rate, and before/after samples

#### Scenario: Adapter fails validation

- **GIVEN** at least one replayed prompt still receives an AST evaluation with `diverged == true`
- **WHEN** validation completes
- **THEN** the validator emits `nebula.validation.failed`
- **AND** the payload includes the adapter artifact, pass rate, and failing samples
