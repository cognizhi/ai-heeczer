# From Tokens to Labor-Equivalent Value: A Mathematical Framework for Translating Agentic AI Telemetry into Human Equivalent Effort and Financial Equivalent Cost

**Author:** @cyyeong  
**Date:** April 23, 2026  
**Document Status:** Research Paper — Draft v1.0  

---

## Abstract

Organizations deploying agentic AI systems accumulate rich operational telemetry —
token counts, task durations, tool call sequences, workflow step counts, retry
patterns, artifact yields — yet lack a principled, auditable method for translating
that telemetry into business-legible value. This paper formalizes the **Human
Equivalent Effort (HEE)** and **Financial Equivalent Cost (FEC)** scoring model
proposed here. We ground the construction of the model in
four convergent research traditions: classical software effort estimation theory
[1, 2, 3], empirical AI productivity studies [4, 5, 6], agentic AI behavioral
research [7, 8, 9], and knowledge-work productivity plus observed productivity
dispersion [10, 14]. We derive the **Base Cognitive Unit (BCU)** as a normalized
effort unit, show how multi-dimensional telemetry signals are combined linearly
into BCU components, define the category and context multiplier system that adjusts
for task semantics and execution quality, and specify the confidence model that
bounds trust in the resulting estimate. We work through a verified end-to-end
numerical example for a representative `code_generation` scenario, and specify a
deterministic arithmetic and rounding contract intended to preserve reproducibility
across future implementations. We discuss the model's limitations, the calibration path, and
directions for future empirical validation.

---

## 1. Introduction

The rise of agentic AI frameworks — LangGraph, Google ADK, PydanticAI, and their
successors — has made it practical to delegate multi-step knowledge work to
autonomous AI agents. These agents can generate code, perform root cause analysis,
draft documents, plan architectures, and execute regulated decision flows. They
generate rich, structured telemetry as a side-effect of execution. What they do not
generate, by default, is any estimate of how much human time or labor cost would
have been consumed to accomplish the same task at the same quality level.

This gap has measurable consequences. Engineering leadership cannot defend AI
investment without business-legible output metrics. Finance teams cannot model
labor-equivalent cost avoidance without a principled translation layer. Platform
teams cannot prioritize improvements without comparing the labor-equivalent value
delivered by different agents and workflows.

Empirical productivity research confirms the magnitude of the underlying effect.
Peng et al. (2023) found that software developers with access to an AI pair
programmer completed an HTTP server implementation task **55.8% faster** than the
control group [4]. Brynjolfsson, Li, and Raymond (2023) found that AI-assisted
customer support workers resolved issues **15% faster on average**, with the largest
gains for less-experienced workers handling unusual problems [6]. Eloundou et al.
(2023) estimated that approximately **80% of the US workforce** may have at least
10% of their work tasks affected by LLM-powered software, and that about 15% of all
worker tasks could be completed significantly faster at the same quality level [5].

Yet the measurement of that productivity impact remains ad hoc. Organizations
accumulate token bills but lack effort-equivalent translations. The goal of the
proposed framework is to close this gap: to provide a deterministic, versioned,
and auditable formula that converts machine-observable telemetry into the unit of
analysis that business strategy requires — human time and its associated labor cost.

This paper proceeds as follows. Section 2 surveys the theoretical foundations.
Section 3 defines the canonical telemetry event schema. Section 4 derives the BCU
model and scoring formula in detail. Section 5 formalizes the confidence model.
Section 6 presents a complete numerical worked example. Section 7 discusses the
multi-tier financial model. Section 8 addresses limitations. Section 9 outlines
calibration and validation directions.

---

## 2. Theoretical Foundations

### 2.1 Classical Software Effort Estimation

Software effort estimation has a four-decade history of translating code
characteristics into human time. The dominant models are:

**COCOMO (Constructive Cost Model).** Boehm (1981) proposed estimating software
development effort as a function of source lines of code (SLOC), adjusted by a set
of cost drivers that capture product complexity, platform constraints, personnel
capability, and project attributes [1]. The general COCOMO formula takes the form:

$$E = a \cdot (\text{KLOC})^b \cdot \prod_{i=1}^{15} f_i$$

where $E$ is effort in person-months, $a$ and $b$ are calibration constants,
and $f_i$ are cost-driver multipliers. The multiplicative structure of COCOMO —
a base measure adjusted by a product of contextual multipliers — is directly
preserved in the HEE model's treatment of category and context multipliers.

**Function Point Analysis.** Albrecht (1979) proposed measuring application
functionality from the user's perspective rather than from implementation size [2].
Function points count inputs, outputs, inquiries, internal files, and external
interfaces, each weighted by a complexity factor and then multiplied by a value
adjustment factor of 14 general system characteristics. This established the
principle that _software work can be quantified by its functional scope and
contextual characteristics_ independent of the implementation language — a principle
that generalizes naturally to AI task telemetry.

**Halstead Complexity Metrics.** Halstead (1977) proposed measuring cognitive effort
in terms of operator and operand vocabulary, showing that program volume correlates
with mental effort [3]. Halstead's volume metric:

$$V = N \cdot \log_2(\eta)$$

where $N$ is the total number of operations and $\eta$ is the total vocabulary size,
presages the token-based component in the BCU model. Just as operator/operand counts
proxy the information-processing burden in a program, token counts proxy the
information-processing burden in an AI-executed task.

The HEE model does not reproduce these classical models literally — AI task
telemetry differs structurally from source code attributes — but it inherits their
two foundational principles: a normalized base unit adjusted by a multiplicative
system of contextual weights.

### 2.2 Knowledge Work Productivity

Drucker (1999) identified the measurement of knowledge-worker productivity as the
defining management challenge of the twenty-first century [10]. Unlike industrial
work, where output per hour is relatively straightforward to measure, knowledge work
output is heterogeneous, quality-dependent, and partially tacit. Drucker's framework
requires that knowledge work be defined by task rather than by input — what matters
is the result produced, not the hours invested.

This framing is foundational to HEE. The question is not "how long did the AI
agent run?" but "how long would it have taken a human of a specified role and skill
level to produce an equivalent result?" The task category, outcome quality, and
contextual risk class are therefore not peripheral annotations — they are primary
determinants of the human-equivalent effort.

### 2.3 AI Productivity Evidence

Three recent empirical studies provide direct calibration anchors for the HEE model.

**Peng et al. (2023)** conducted a controlled experiment with GitHub Copilot, in
which recruited developers were asked to implement an HTTP server in JavaScript as
quickly as possible. The AI-assisted group completed the task in a median time
**55.8% shorter** than the control group [4]. This provides a direct empirical
lower bound: under favorable conditions, AI assistance at least doubles the pace
of knowledge work completion for well-defined coding tasks.

**Brynjolfsson, Li, and Raymond (2023)** studied 5,172 customer support agents
given access to a generative AI assistant. Productivity, measured as issues resolved
per hour, increased by **15% on average**, with the largest gains (speed and
quality simultaneously) for less-experienced workers handling rare problem types [6].
The heterogeneous effect — junior workers gain most — is reflected in the HEE model's
tier-adjustment step: the same AI-executed task represents more labor-equivalent
savings relative to a junior role than relative to a principal engineer whose
baseline productivity is already very high.

**Eloundou et al. (2023)** applied a rubric to assess which US worker tasks are
exposed to LLM-powered software, concluding that **approximately 15% of all worker
tasks** could be completed significantly faster at the same quality level with LLM
access, rising to 47–56% when LLM-powered tooling is included [5]. They further
found that higher-income, higher-skill roles face greater fractional exposure — which
justifies the HEE model's use of a productivity multiplier that scales with seniority.

**Drori et al. (2022)** demonstrated the time-translation phenomenon directly in an
academic context: machine learning final exam questions that take faculty **days**
to write and students **hours** to solve are handled by large language models in
**seconds** [11]. The title "From Human Days to Machine Seconds" captures the
orders-of-magnitude translation that the HEE model formalizes at the task level.

### 2.4 Agentic AI Systems and Multi-Signal Telemetry

Modern agentic AI systems do not simply run inference once. They plan, iterate,
call external tools, execute multi-step workflows, and produce structured artifacts.
This multi-step character is documented in the SWE-agent work (Yang et al., 2024),
which showed that the design of the agent-computer interface — controlling tool
availability, file editing capacity, and repository navigation — dramatically
affects the agent's task completion rate on the SWE-bench benchmark of real GitHub
issues [7]. Each of these agentic behaviors — tool calls, workflow steps, artifact
production — generates distinct telemetry that carries independent signal about the
complexity and cognitive scope of the task.

The MINT benchmark (Wang et al., 2023) demonstrated that LLM performance on
complex reasoning tasks improves by **1–8% per tool use turn** and by **2–17% with
natural language feedback**, confirming that tool use and multi-turn interaction are
reliable indicators of task complexity [9]. These empirical findings justify the
inclusion of tool call count and workflow step count as distinct BCU components,
rather than treating them as redundant with task duration alone.

---

## 3. The Canonical Telemetry Event

The HEE/FEC model is defined over a canonical JSON event (schema version 1.0).
The excerpt below retains the fields consumed directly by validation and scoring;
additional deployment-specific metadata may be added in future operational
settings without altering the core scoring inputs.

```json
{
  "spec_version": "1.0",
  "event_id":     "<uuid-v4>",
  "timestamp":    "2026-04-22T09:46:00Z",
  "task": {
    "name":     "generate_api_spec",
    "category": "code_generation",
    "outcome":  "success"
  },
  "metrics": {
    "duration_ms":         14500,
    "tokens_prompt":        1200,
    "tokens_completion":    4000,
    "tool_call_count":         3,
    "workflow_steps":          5,
    "retries":                 1,
    "artifact_count":          4,
    "output_size_proxy":     2.5
  },
  "context": {
    "human_in_loop":    false,
    "review_required":  true,
    "temperature":       0.2,
    "risk_class":      "medium"
  },
  "identity": {
    "tier_id": "mid_level_engineer"
  }
}
```

Fields not consumed directly by scoring (for example workspace, project,
framework-source, and extension metadata) are omitted here for brevity.

### 3.1 Normalization Contract

Before scoring, the event is normalized according to a strict deterministic
contract proposed for any implementation of the model:

| Field | Missing value rule |
|---|---|
| `task.category` | → `"uncategorized"`; incurs confidence penalty |
| `tokens_prompt`, `tokens_completion` | → `0`; incurs confidence penalty |
| `workflow_steps` | → `0`; incurs confidence penalty |
| `tool_call_count` | → `0`; incurs confidence penalty |
| `retries` | → `0` |
| `artifact_count` | → `0` |
| `output_size_proxy` | → `0.0` |
| `review_required` | → `false` |
| `human_in_loop` | → `false` |
| `temperature` | → `0.0` |
| `risk_class` | → `Medium` |
| Required missing fields | → Validation error; no score produced |

Required fields that may not be inferred include `event_id`, `timestamp`,
`task.name`, `task.outcome`, and `metrics.duration_ms`. Their absence fails
validation rather than triggering a fallback.

---

## 4. The BCU Scoring Model

### 4.1 Base Cognitive Unit: Definition

The **Base Cognitive Unit (BCU)** is a normalized effort unit defined such that
**1 BCU ≈ 1 baseline human minute** of knowledge work for a mid-level role before
any role-specific or contextual adjustment. BCU is not directly observable; it is
a computed intermediate quantity that aggregates multiple independent telemetry
signals into a single scalar representing the raw cognitive scope of the task.

The design is deliberately linear in the component signals. Linearity is justified
on two grounds. First, the empirical evidence for non-linear interactions between
individual effort signals in AI task contexts is thin; a linear model is more
auditable, more explainable, and more resistant to calibration gaming. Second, the
components capture structurally independent aspects of task work (language
processing burden, temporal commitment, procedural complexity, tool orchestration,
output production, and review burden) and additive combination is the natural prior
when interactions are unknown.

### 4.2 BCU Component Formulas

Let the normalized event produce the following scalar values (all non-negative):

| Symbol | Source |
|---|---|
| $T$ | `tokens_prompt + tokens_completion` |
| $D$ | `duration_ms / 1000` (seconds) |
| $W$ | `workflow_steps` |
| $U$ | `tool_call_count` |
| $A$ | `min(artifact_count, artifact_cap)` |
| $P$ | `output_size_proxy` |
| $R$ | `review_required` (boolean → weight if true, else 0) |

The BCU is computed as:

$$\text{BCU} = C_T + C_D + C_W + C_U + C_A + C_P + C_R$$

where the individual components are:

$$C_T = \frac{T}{\delta_T}$$

$$C_D = \frac{D}{\delta_D}$$

$$C_W = W \cdot w_W$$

$$C_U = U \cdot w_U$$

$$C_A = A \cdot w_A$$

$$C_P = P \cdot \omega_{P,\kappa}$$

$$C_R = \begin{cases} \rho_\kappa & \text{if } review\_required \\ 0 & \text{otherwise} \end{cases}$$

where:

- $\delta_T$ = token divisor (default: 500)
- $\delta_D$ = duration-seconds divisor (default: 2)
- $w_W$ = step weight (default: 2)
- $w_U$ = tool weight (default: 3)
- $w_A$ = artifact weight (default: 1.5)
- $\omega_{P,\kappa}$ = per-category output weight
- $\rho_\kappa$ = per-category review weight
- $\kappa$ = task category

The literature supports the inclusion of these signals, but it does not identify
the exact coefficients proposed in Appendix A. Those numerical coefficients should
therefore be interpreted as **reference priors** for an initial scoring model,
not as universal empirical constants. The empirical literature supports signal
selection; calibration against local ground truth is what turns those defaults
into organization-specific estimates.

**Justification of default weights.** Each default parameter value corresponds to
an approximate modeling role in the reference profile proposed here:

- **Token divisor = 500.** This is the normalization unit used to scale token mass
  into the BCU range. It makes token volume a material contributor without letting
  raw token counts dominate every score. Because token density varies sharply by
  task and modality, the divisor is best treated as a calibration prior rather than
  a claim that 500 tokens universally equal one human minute.

- **Duration divisor = 2.** This sets elapsed runtime as a meaningful but bounded
  proxy for task scope. In the reference profile, duration contributes materially to
  the score while remaining commensurate with higher-level orchestration signals
  such as steps, tool calls, and review burden.

- **Step weight = 2.** Multi-step execution is a core feature of agentic systems,
  and the literature supports workflow decomposition as a real signal of task
  complexity [7, 9]. The specific coefficient of 2 is the proposed initial prior
  for converting that signal into BCU.

- **Tool weight = 3.** Tool-using agents outperform single-shot inference on many
  complex tasks [7, 9], so tool-call count is treated as an independent complexity
  indicator. The coefficient of 3 is a proposed initial default rather than a
  value directly estimated in the cited studies.

- **Artifact weight = 1.5, capped at 20.** Each artifact (file created, PR
  opened, ticket filed, report generated) captures delivered output volume. The cap
  is a boundedness guardrail: it prevents bulk generation events (for example,
  template expansion that produces 1,000 files) from dominating the score and
  makes gaming the model materially harder.

### 4.3 Category Multiplier

BCU is a category-agnostic quantity. The _cognitive density_ of different task
categories differs significantly. Producing a structured summary of a document
(summarization) involves primarily linear comprehension and extraction. Performing
root cause analysis over distributed traces involves hypothesis generation, causal
inference, and iterative validation — qualitatively more demanding per unit of
output.

The category multiplier $\mu_\kappa$ scales the total BCU for the semantic class
of the task:

$$\text{BCU}_\text{adjusted} = \text{BCU} \cdot \mu_\kappa$$

Default category multipliers (v1.0):

| Category | $\mu_\kappa$ | Justification |
|---|---|---|
| `summarization` | 0.9 | Largely extraction; lower cognitive density than generation |
| `drafting` | 1.0 | Reference level; open-ended but bounded |
| `code_generation` | 1.2 | Requires precision, correctness constraints, integration awareness |
| `root_cause_analysis` | 1.4 | Hypothesis generation, causal reasoning, evidence synthesis |
| `planning_architecture` | 1.5 | Design decisions with long-horizon consequences |
| `regulated_decision_support` | 1.6 | Compliance constraints, interpretability requirements, audit trails |
| `uncategorized` | 1.0 | Neutral fallback; incurs confidence penalty |

The category multiplier design draws on the concept of cost-driver multipliers
from COCOMO [1], where each cost driver reflects a structural property of the work
that affects the total effort required beyond what the base metric captures. The
numeric defaults listed above are proposed baseline values.

### 4.4 Contextual Multiplier

The contextual multiplier $\Gamma$ adjusts for execution quality and operational
conditions. It is defined as a product of five independent factors:

$$\Gamma = \gamma_\text{retry} \cdot \gamma_\text{ambiguity} \cdot \gamma_\text{risk} \cdot \gamma_\text{hil} \cdot \gamma_\text{outcome}$$

**Retry multiplier.** Failed or retried attempts impose additional cognitive cost on
human interpreters and downstream integrators. Each retry represents a failure of
the task to converge on first execution, which in human workflows would typically
involve re-specification, re-execution, and re-validation:

$$\gamma_\text{retry} = \min\!\left(1 + r \cdot \gamma_r,\ \Gamma_\text{max}\right)$$

where $r$ is the retry count, $\gamma_r = 0.25$ is the per-retry increment, and
$\Gamma_\text{max} = 2.0$ is the cap. The cap prevents pathological retry loops
from producing arbitrarily inflated scores.

**Ambiguity multiplier.** Temperature is a proxy for generation randomness and task
underspecification. Higher temperature implies that the task was underspecified
enough to admit multiple plausible outputs, which corresponds to genuine ambiguity
in the human-task specification:

$$\gamma_\text{ambiguity} = \begin{cases} 1.1 & \text{if } \tau > 0.7 \\ 1.0 & \text{otherwise} \end{cases}$$

where $\tau$ is the generation temperature. The threshold of 0.7 is proposed as a
practical baseline for separating lower-variance from exploratory generation
modes.

**Risk multiplier.** Tasks executed under high-risk conditions require more careful
output verification, downstream validation, and documentation by human reviewers:

$$\gamma_\text{risk} = \begin{cases} 1.2 & \text{if risk\_class = High} \\ 1.0 & \text{otherwise} \end{cases}$$

**Human-in-loop multiplier.** When substantial human review accompanies the AI
execution, the effective AI-equivalent contribution is lower — humans absorb a
meaningful share of the cognitive burden. This mirrors the COCOMO "analyst
capability" driver:

$$\gamma_\text{hil} = \begin{cases} 0.7 & \text{if } human\_in\_loop = \text{true} \\ 1.0 & \text{otherwise} \end{cases}$$

**Outcome multiplier.** Task outcome is the primary quality gate. A failed task
produces far less realized value than a successful one, even if it consumed the same
execution resources:

$$\gamma_\text{outcome} = \begin{cases} 1.0 & \text{success} \\ 0.6 & \text{partial\_success} \\ 0.25 & \text{failure} \\ 0.2 & \text{timeout} \end{cases}$$

These outcome weights are calibrated to reflect that partial success retains
meaningful but incomplete value, while failure and timeout produce near-negligible
realized benefit. As with the other context coefficients, they are proposed
baseline defaults rather than coefficients fitted from an external benchmark.

### 4.5 Baseline Human Minutes

The **baseline human minutes** $H_0$ is the pre-tier estimate of human-equivalent
effort at the reference (mid-level) productivity level:

$$H_0 = \text{BCU} \cdot \mu_\kappa \cdot \Gamma$$

$H_0$ is the model's core output before financial adjustment. It represents the
number of minutes a mid-level knowledge worker in the same role category would
typically require to produce an equivalent result.

### 4.6 Tier Adjustment

Different role levels operate at different productivity rates. A principal engineer
can produce a high-quality architecture design in 30 minutes that a junior engineer
might take 90 minutes to produce at equivalent quality. AI systems do not change
roles, but the choice of comparison tier determines the labor-equivalent value
interpretation.

Let $\pi_t$ be the productivity multiplier for tier $t$. The **tier-adjusted effort
estimate** is:

$$H_t = \frac{H_0}{\pi_t}$$

Default tier definitions:

| Tier | $\pi_t$ | Hourly Rate (USD) |
|---|---|---|
| Principal Engineer | 3.0 | 200 |
| Senior Engineer | 2.0 | 125 |
| Mid-Level Engineer | 1.0 | 75 |
| Junior Engineer | 0.5 | 45 |

The productivity multiplier represents a relative multiplier normalized to the
mid-level as the reference. $\pi = 2.0$ for a senior engineer means that a senior
engineer would complete the same task in half the time of a mid-level engineer.
This gives the following relationship:

$$H_\text{senior} = \frac{H_0}{2.0}$$

That is, a principal engineer would complete the equivalent task in one-third of
the mid-level time. This is consistent with empirical estimates of programmer
productivity variance: Sackman, Erickson, and Grant (1968) measured 10:1
productivity variation between best and worst programmers; the 6:1 ratio implied
by the proposed tier defaults (principal vs. junior; $\pi = 3.0$ vs. $\pi = 0.5$)
is directionally consistent with that dispersion [14], but remains a modeling
choice rather than a parameter fitted directly from the Sackman study.

The final output quantities are:

$$H_t^\text{(min)} = \frac{H_0}{\pi_t} \quad \text{(estimated minutes)}$$

$$H_t^\text{(hr)} = \frac{H_t^\text{(min)}}{60} \quad \text{(estimated hours)}$$

$$H_t^\text{(day)} = \frac{H_t^\text{(hr)}}{h_t} \quad \text{(estimated days)}$$

where $h_t$ is the working hours per day for tier $t$ (default 8).

---

## 5. Financial Equivalent Cost

### 5.1 FEC Formula

The **Financial Equivalent Cost (FEC)** converts the tier-adjusted effort estimate
into a labor-equivalent monetary value:

$$\text{FEC}_t = H_t^\text{(hr)} \cdot \lambda_t$$

where $\lambda_t$ is the hourly rate for tier $t$ in the configured currency.

FEC is explicitly labeled a _labor-equivalent estimate_, not an accounting figure.
It answers the question: "If this task had been delegated to a human of role $t$,
what would the labor cost have been?" It does not represent payroll cost, billing
rate, or contract value directly.

### 5.2 Currency and Multi-Tenancy

In operational use, labor rates should be parameterized by organization,
currency, effective date, and role definition so the resulting estimates remain
auditable when economic assumptions change over time.

---

## 6. Confidence Model

### 6.1 Motivation

The BCU model cannot distinguish between a high-quality 5,000-token task and a
low-quality 5,000-token task from token count alone. Nor can it know whether a
12-second duration reflects 12 seconds of meaningful reasoning or 12 seconds of
retrying a broken tool call. The confidence model quantifies how trustworthy a
given score estimate is, based on the completeness of the telemetry signals
provided.

A score with confidence 0.95 reflects full telemetry and a declared category under
the proposed baseline profile.
A score with confidence 0.35 reflects sparse telemetry (duration only, no tokens,
no steps, no tools) against an uncategorized task. Both numbers are correct given
their inputs; the confidence score is the instrument by which users understand
how much to trust each one.

### 6.2 Confidence Formula

Starting from a base confidence value $\beta$ (default: 0.95), penalties are
subtracted for missing telemetry signals and retries:

$$c_{raw} = \beta - \pi_\kappa - \pi_T - \pi_W - \pi_U - \pi_r$$

$$c_{risk} = \begin{cases} \min(c_{raw}, \text{cap}_{HR}) & \text{if risk\_class = High} \\ c_{raw} & \text{otherwise} \end{cases}$$

$$c = \text{clamp}(c_{risk}, c_\text{min}, c_\text{max})$$

where:

| Symbol | Condition | Default penalty |
|---|---|---|
| $\pi_\kappa$ | `task.category` was missing | −0.10 |
| $\pi_T$ | both token fields were missing | −0.15 |
| $\pi_W$ | `workflow_steps` was missing | −0.05 |
| $\pi_U$ | `tool_call_count` was missing | −0.05 |
| $\pi_r$ | per-retry penalty, capped at $P_\text{cap}$ | $\min(r \cdot 0.05,\ 0.20)$ |
| $\text{cap}_{HR}$ | high-risk cap | if `risk_class = High` and the pre-clamp score exceeds 0.85, cap it at 0.85 |

The clamp enforces $c \in [0.0, 1.0]$.
The high-risk term is therefore an upper bound, not an additive subtraction.

**Confidence bands** map the raw score to an ordinal label:

| Band | Range |
|---|---|
| High | $[0.85, 1.00]$ |
| Medium | $[0.60, 0.84]$ |
| Low | $[0.40, 0.59]$ |
| Very Low | $[0.00, 0.39]$ |

The band derivation uses the _unrounded_ confidence score, consistent with the
principle that display formatting must not affect the categorical interpretation of
the estimate.

---

## 7. Rounding and Determinism Contract

Any implementation of the model should use fixed-point decimal computation with at
least 4 fractional digits for intermediate steps. This prevents floating-point
accumulation errors from diverging across implementations. The final rounding rule
is **round half away from zero** applied to:

| Output | Decimal places |
|---|---|
| `final_estimated_minutes` | 2 |
| `estimated_hours` | 2 |
| `estimated_days` | 2 |
| `financial_equivalent_cost` | 2 |
| `confidence_score` | 4 |

The `confidence_band` is derived from the unrounded intermediate value before the
`confidence_score` field is rounded.

---

## 8. End-to-End Worked Example

We reproduce the complete numerical derivation for a reference
`code_generation` scenario under the baseline profile and the mid-level
engineering tier.

### 8.1 Input

```text
tokens_prompt     = 1,200
tokens_completion = 4,000
duration_ms       = 14,500
workflow_steps    = 5
tool_call_count   = 3
artifact_count    = 4
output_size_proxy = 2.5
review_required   = true
human_in_loop     = false
temperature       = 0.2
risk_class        = medium
retries           = 1
category          = code_generation
outcome           = success
tier              = Mid-Level Engineer  (π = 1.0, λ = $75/hr)
```

### 8.2 Normalization

```text
total_tokens     = 1,200 + 4,000 = 5,200
duration_seconds = 14,500 / 1,000 = 14.5
```

No fields are missing; no confidence penalties from missingness.

### 8.3 BCU Component Computation

Using the proposed baseline weights:

$$C_T = \frac{5200}{500} = 10.40 \text{ BCU}$$

$$C_D = \frac{14.5}{2} = 7.25 \text{ BCU}$$

$$C_W = 5 \times 2 = 10.00 \text{ BCU}$$

$$C_U = 3 \times 3 = 9.00 \text{ BCU}$$

$$C_A = \min(4, 20) \times 1.5 = 4 \times 1.5 = 6.00 \text{ BCU}$$

For the output component, the `code_generation` category output weight is 1.2:

$$C_P = 2.5 \times 1.2 = 3.00 \text{ BCU}$$

For the review component, the `code_generation` category review weight is 5:

$$C_R = 5 \text{ BCU} \quad (\text{since } review\_required = \text{true})$$

$$\text{BCU} = 10.40 + 7.25 + 10.00 + 9.00 + 6.00 + 3.00 + 5.00 = \mathbf{50.65}$$

### 8.4 Category Multiplier

$$\mu_\kappa = 1.2 \quad (\text{code\_generation})$$

### 8.5 Context Multiplier

$$\gamma_\text{retry} = \min(1 + 1 \times 0.25,\ 2.0) = 1.25$$

$$\gamma_\text{ambiguity} = 1.0 \quad (\tau = 0.2 \leq 0.7)$$

$$\gamma_\text{risk} = 1.0 \quad (\text{medium risk})$$

$$\gamma_\text{hil} = 1.0 \quad (human\_in\_loop = \text{false})$$

$$\gamma_\text{outcome} = 1.0 \quad (\text{success})$$

$$\Gamma = 1.25 \times 1.0 \times 1.0 \times 1.0 \times 1.0 = 1.25$$

### 8.6 Baseline Human Minutes

$$H_0 = 50.65 \times 1.2 \times 1.25 = 50.65 \times 1.5 = 75.975 \text{ minutes}$$

Rounded to 2 d.p.: $H_0 = \mathbf{75.98}$ minutes.

### 8.7 Tier Adjustment

For the Mid-Level Engineer tier, $\pi = 1.0$:

$$H_t^\text{(min)} = \frac{75.975}{1.0} = 75.975 \approx \mathbf{75.98} \text{ min}$$

$$H_t^\text{(hr)} = \frac{75.975}{60} = 1.26625 \approx \mathbf{1.27} \text{ hr}$$

$$H_t^\text{(day)} = \frac{1.26625}{8} = 0.1582...\approx \mathbf{0.16} \text{ days}$$

### 8.8 Financial Equivalent Cost

$$\text{FEC} = 1.26625 \times 75 = 94.969\ldots \approx \mathbf{\$94.97}$$

### 8.9 Confidence Score

Starting from base $\beta = 0.95$:

- No missing category → no penalty
- No missing tokens → no penalty
- No missing steps → no penalty
- No missing tools → no penalty
- Retry penalty: $1 \times 0.05 = 0.05$

$$c = 0.95 - 0.05 = 0.90$$

High-risk cap does not apply (risk is medium). Final confidence: $\mathbf{0.9000}$.

Confidence band: $0.90 \geq 0.85$ → **High**.

### 8.10 Verified Output

The following serialized result illustrates the proposed output shape for the
reference scenario:

```json
{
  "spec_version":    "1.0",
  "bcu_breakdown": {
    "tokens":   "10.40",
    "duration": "7.25",
    "steps":    "10",
    "tools":    "9",
    "artifacts":"6.0",
    "output":   "3.00",
    "review":   "5"
  },
  "category":              "code_generation",
  "category_multiplier":   "1.2",
  "context_multiplier": {
    "retry":        "1.25",
    "ambiguity":    "1",
    "risk":         "1.0",
    "human_in_loop":"1",
    "outcome":      "1.0"
  },
  "baseline_human_minutes":    "75.98",
  "tier": {
    "name":        "Mid-Level Engineer",
    "multiplier":  "1.0",
    "hourly_rate": "75",
    "currency":    "USD"
  },
  "final_estimated_minutes":   "75.98",
  "estimated_hours":           "1.27",
  "estimated_days":            "0.16",
  "financial_equivalent_cost": "94.97",
  "confidence_score":          "0.9000",
  "confidence_band":           "High",
  "human_summary": "Estimated 75.98 Mid-Level Engineer-equivalent minutes (~94.97 cost) for `code_generation`; confidence high."
}
```

All values match the step-by-step derivation above, confirming that the symbolic
formula and the serialized example are internally consistent.

---

## 9. Discussion

### 9.1 The Token Divisor as a Calibration Parameter

The token divisor of 500 is a calibration constant, not a universal truth. Its
purpose in the proposed baseline profile is to keep token volume commensurate with
duration, steps, and tools while preserving the interpretation that 1 BCU is
approximately 1 baseline human minute. It should therefore be treated as a tunable
prior requiring task-level calibration, not as an invariant conversion from tokens
to labor time.

### 9.2 Additivity of BCU Components

The linear additive BCU formula treats the component signals as independent
contributors to effort. This is an approximation. In practice, a task with many
tool calls and many workflow steps is likely highly correlated — both are proxies
for orchestration complexity — and a strictly additive model may over-count. Future
work should investigate whether interaction terms or a nonlinear combination of
high-correlation components improves calibration against human baselines.

### 9.3 The Human-in-Loop Discount

The 0.7 multiplier for `human_in_loop = true` reflects that the AI's contribution
to a human-in-the-loop workflow is partial. This is consistent with the
Brynjolfsson et al. (2023) finding that AI assistance raises productivity but does
not eliminate the human role in quality-sensitive workflows [6]. The exact discount
factor requires calibration per task category. For regulated decision support, human
review may account for 50–70% of the total cognitive burden; for code generation
with reviewer sign-off, the fraction may be lower.

### 9.4 Outcome Multipliers and Realized Value

The outcome multipliers (1.0 / 0.6 / 0.25 / 0.2) encode the principle that
AI-executed tasks have realized value only proportional to their quality of
completion. A failed task that triggered human escalation may have _negative_
realized value (net effort increase). The current model does not model negative
value; it floors at 0.2 to represent that even a timeout may have diagnostic value.
Organizations with data on failure modes should calibrate these weights from
empirical outcomes; until then they remain proposed baseline defaults.

### 9.5 Category Multipliers and Cognitive Science

The ordering of category multipliers (summarization < drafting < code_generation <
root_cause_analysis < planning_architecture < regulated_decision_support) reflects
the cognitive hierarchy from comprehension-and-extraction tasks toward
synthesis-and-judgment tasks. This ordering is consistent with Bloom's Taxonomy of
cognitive complexity [12], where analysis, evaluation, and creation represent
higher-order cognitive operations than remembering and understanding.

### 9.6 Computational Reproducibility

The determinism contract — fixed-point arithmetic, explicit fallback rules, round-
half-away-from-zero, 4 d.p. intermediate precision — is required because binary
floating-point representations of the same decimal arithmetic can diverge across
execution environments. Any serious implementation of the model should therefore
use decimal or fixed-point arithmetic and verify that the reference examples
produce the same rounded results regardless of runtime.

---

## 10. Calibration and Validation

### 10.1 Calibration Methodology

The baseline profile parameters (token divisor, component weights, category
multipliers, tier multipliers, confidence penalties) are initial calibration values
derived from:

1. The foundational effort estimation literature [1, 2, 3]
2. Published AI productivity studies [4, 5, 6]
3. Domain knowledge of software engineering and knowledge work norms

They are not empirically validated against ground-truth human effort measurements
in a controlled experiment. Organizations adopting this model should treat the
baseline profile as a starting point and calibrate using the following methodology:

1. **Select a reference task set.** Choose 10–50 tasks with known human effort
   (measured from time-tracking systems, sprint logs, or retrospective estimates).
2. **Submit equivalent AI executions** through the proposed telemetry translation
  pipeline and record the HEE
   output for each task.
3. **Compute calibration residuals.** For each task, measure
   $\epsilon_i = H_\text{HEE,i} - H_\text{human,i}$.
4. **Adjust profile parameters** to minimize $\sum_i \epsilon_i^2$ subject to the
   constraint that no parameter violates its documented safety bounds.
5. **Publish and version the calibrated parameter set** so downstream reporting
  can be reproduced against a known profile revision.
6. **Recompute the reference examples and parity cases** to confirm that the
  calibrated profile preserves deterministic arithmetic and documented rounding
  behavior.

A credible calibration study should also reserve a hold-out validation subset and
report absolute error and signed bias by category and tier, so tuned coefficients
are not evaluated only on the tasks used to fit them.

### 10.2 Limitations of the Current Model

1. **No empirical validation on production datasets.** The default weights are
   calibrated from literature and domain knowledge, not from a controlled trial.
2. **Token counts are a proxy for semantic content.** A task that generates 4,000
   tokens of boilerplate code and one that generates 4,000 tokens of algorithmic
   design may represent very different human-equivalent effort.
3. **Output size proxy is coarse.** The `output_size_proxy` field is a scalar
   (e.g., number of pages, number of artifacts weighted by type). A richer measure
   (e.g., functional complexity of code, argument depth of a document) would improve
   resolution.
4. **No task-specific benchmarks are included yet.** The calibration pack
   (Section 10.1) is defined but not populated with empirical measurements.
5. **Framework heterogeneity.** Different agentic frameworks emit telemetry
   differently. The canonical schema normalizes inputs, but adapter quality varies.

### 10.3 Future Work

- Empirical calibration study using production AI workloads from multiple
  organizations, with paired human effort estimates.
- Investigation of interaction terms between BCU components for high-correlation
  signal pairs (steps × tools, tokens × duration).
- Extension of the confidence model to include _calibration signal_: tasks in
  categories with calibrated profiles should receive a confidence bonus.
- Per-category token divisor calibration, recognizing that code-generation tokens
  are semantically denser than summarization tokens.
- Time-series analysis of HEE/FEC trends to detect model drift as AI capabilities
  evolve.

---

## 11. Conclusion

We have presented a complete mathematical derivation of the HEE and FEC scoring
model proposed in this paper. The model translates multi-signal agentic AI
telemetry — token counts, task duration, workflow steps, tool call counts, artifact
production, output volume, and contextual flags — into a deterministic estimate of
human-equivalent effort in minutes, hours, and days, along with a labor-equivalent
financial cost.

The model is grounded in four convergent research traditions: classical software
effort estimation (COCOMO, Function Points, Halstead metrics), knowledge work
productivity theory (Drucker), empirical AI productivity studies (Copilot, customer
support AI, LLM task exposure), and agentic AI behavioral research (SWE-agent, MINT).

The construction choices — linear BCU aggregation, multiplicative category and
context adjustments, tier-based productivity scaling, deterministic fixed-point
arithmetic, and a telemetry-completeness confidence model — are individually
justified by the evidence base and collectively produce a system that is auditable,
configurable, versioned, and suitable for reproducible implementation across
multiple language environments.

The reference worked example demonstrates a complete 26-step numerical derivation
producing a concrete output: **75.98 mid-level-engineer-equivalent minutes,
$94.97 FEC, confidence 0.9000 (High)** for a code generation task with full
telemetry. The derivation is transparent enough to be verified by hand, and any
subsequent implementation should preserve these exact rounded results for the
reference scenario.

The model is not a finished empirical instrument. Its default parameters require
calibration against real-world human effort baselines. Its limitations are
explicitly documented. Its confidence model quantifies rather than hides the
uncertainty in every estimate. These properties — honesty about assumptions,
auditability of computations, configurability of parameters — are what distinguish
a principled analytics framework from an ad hoc metric.

---

## References

**[1]** Boehm, B. W. (1981). _Software Engineering Economics_. Prentice-Hall,
Englewood Cliffs, NJ. ISBN 0-13-822122-7.  
The foundational treatise on constructive cost modeling. Introduced the COCOMO
model family and the use of multiplicative cost drivers to adjust a base effort
estimate. The structure of the HEE model's contextual multiplier system is directly
inspired by COCOMO's cost-driver product.

**[2]** Albrecht, A. J. (1979). Measuring application development productivity. In
_Proceedings of the IBM Application Development Symposium_, Monterey, CA, October
1979 (pp. 83–92). IBM Corporation.  
Introduced Function Point Analysis, establishing the principle that software work
can be quantified from its functional scope and user-visible complexity rather than
from implementation size. Justifies the per-component weighting approach in BCU.

**[3]** Halstead, M. H. (1977). _Elements of Software Science_. Elsevier North-
Holland, New York. ISBN 0-444-00205-7.  
Proposed the first information-theoretic software complexity metrics, including
program volume $V = N \log_2 \eta$. The analogy between operator/operand counts as
a proxy for cognitive burden and token counts as a proxy for information processing
is a direct conceptual descendant of Halstead's framework.

**[4]** Peng, S., Kalliamvakou, E., Cihon, P., and Demirer, M. (2023). The Impact
of AI on Developer Productivity: Evidence from GitHub Copilot. _arXiv preprint_
arXiv:2302.06590. [https://arxiv.org/abs/2302.06590](https://arxiv.org/abs/2302.06590)  
> "The treatment group, with access to the AI pair programmer, completed the task
> 55.8% faster than the control group."  
Provides the primary controlled experimental evidence that AI assistance produces
measurable, large productivity gains on bounded software development tasks.

**[5]** Eloundou, T., Manning, S., Mishkin, P., and Rock, D. (2023). GPTs are GPTs:
An Early Look at the Labor Market Impact Potential of Large Language Models.
_arXiv preprint_ arXiv:2303.10130. [https://arxiv.org/abs/2303.10130](https://arxiv.org/abs/2303.10130)  
> "Around 80% of the U.S. workforce could have at least 10% of their work tasks
> affected by the introduction of LLMs, while approximately 19% of workers may
> see at least 50% of their tasks impacted."  
> "With access to an LLM, about 15% of all worker tasks in the US could be
> completed significantly faster at the same level of quality."  
Provides macro-level labor market exposure estimates for LLM capabilities, with
specific findings on skill-level exposure that motivate the tiered productivity
multiplier structure in HEE.

**[6]** Brynjolfsson, E., Li, D., and Raymond, L. (2023). Generative AI at Work.
_arXiv preprint_ arXiv:2304.11771. [https://arxiv.org/abs/2304.11771](https://arxiv.org/abs/2304.11771)  
> "Access to AI assistance increases worker productivity, as measured by issues
> resolved per hour, by 15% on average, with substantial heterogeneity across
> workers. Less experienced and lower-skilled workers improve both the speed and
> quality of their output while the most experienced and highest-skilled workers
> see small gains in speed and small declines in quality."  
Provides the strongest direct empirical evidence for the heterogeneous productivity
effect that motivates the HEE tier-adjustment step: the same AI-executed task
represents different amounts of labor-equivalent savings for different skill levels.

**[7]** Yang, J., Jimenez, C. E., Wettig, A., Lieret, K., Yao, S., Narasimhan, K.,
and Press, O. (2024). SWE-agent: Agent-Computer Interfaces Enable Automated
Software Engineering. _arXiv preprint_ arXiv:2405.15793.
[https://arxiv.org/abs/2405.15793](https://arxiv.org/abs/2405.15793)  
> "LM agents represent a new category of end users with their own needs and
> abilities, and would benefit from specially-built interfaces to the software
> they use."  
Establishes the multi-step agentic execution model as the operative unit of analysis
and demonstrates that tool use and interface design are primary determinants of
AI task completion quality — justifying their inclusion as BCU components.

**[8]** Jin, H., Huang, L., Cai, H., Yan, J., Li, B., and Chen, H. (2024). From
LLMs to LLM-based Agents for Software Engineering: A Survey of Current, Challenges
and Future. _arXiv preprint_ arXiv:2408.02479.
[https://arxiv.org/abs/2408.02479](https://arxiv.org/abs/2408.02479)  
Surveys the LLM-agent literature across requirement engineering, code generation,
autonomous decision-making, software design, test generation, and maintenance —
providing the task taxonomy that informed the HEE category multiplier design.

**[9]** Wang, X., Wang, Z., Liu, J., Chen, Y., Yuan, L., Peng, H., and Ji, H.
(2023). MINT: Evaluating LLMs in Multi-turn Interaction with Tools and Language
Feedback. _arXiv preprint_ arXiv:2309.10691. ICLR 2024.
[https://arxiv.org/abs/2309.10691](https://arxiv.org/abs/2309.10691)  
> "LLMs generally benefit from tools and language feedback, with performance gains
> (absolute) of 1–8% for each turn of tool use and 2–17% with natural language
> feedback."  
Provides empirical evidence that tool use count and multi-turn interaction are
reliable signals of task complexity, justifying the tool call count as an
independent BCU component.

**[10]** Drucker, P. F. (1999). Knowledge-Worker Productivity: The Biggest
Challenge. _California Management Review_, 41(2), 79–94.
[https://doi.org/10.2307/41165987](https://doi.org/10.2307/41165987)  
The foundational management paper on knowledge work productivity. Drucker's central
claim — that knowledge work must be defined by task output and its quality, not
by hours of input — is the conceptual foundation of the HEE model's outcome
multiplier and task-category structure.

**[11]** Drori, I., Zhang, S. J., Shuttleworth, R., et al. (2022). From Human Days
to Machine Seconds: Automatically Answering and Generating Machine Learning Final
Exams. _arXiv preprint_ arXiv:2206.05442.
[https://arxiv.org/abs/2206.05442](https://arxiv.org/abs/2206.05442)  
> "A final exam in machine learning at a top institution typically takes faculty
> days to write, and students hours to solve. We demonstrate that large language
> models pass machine learning finals at a human level ... in seconds."  
Demonstrates the orders-of-magnitude time translation between human and AI
execution of the same cognitive task — the core phenomenon that HEE quantifies.

**[12]** Bloom, B. S., Engelhart, M. D., Furst, E. J., Hill, W. H., and Krathwohl,
D. R. (1956). _Taxonomy of Educational Objectives: The Classification of
Educational Goals. Handbook I: Cognitive Domain_. David McKay Company, New York.  
The foundational cognitive taxonomy, organizing cognitive operations from
remembering through understanding, applying, analyzing, evaluating, and creating.
The ordering of HEE category multipliers (summarization → regulated decision
support) follows this cognitive hierarchy.

**[13]** Zhang, Z., Yao, Y., Zhang, A., Tang, X., Ma, X., He, Z., Wang, Y.,
Gerstein, M., and Wang, R. (2023). Igniting Language Intelligence: The Hitchhiker's
Guide From Chain-of-Thought Reasoning to Language Agents. _arXiv preprint_
arXiv:2311.11797. [https://arxiv.org/abs/2311.11797](https://arxiv.org/abs/2311.11797)  
Survey of the language agent literature from chain-of-thought reasoning through
multi-step planning and tool use. Provides theoretical grounding for treating
workflow step count as a measure of agentic task complexity.

**[14]** Sackman, H., Erickson, W. J., and Grant, E. E. (1968). Exploratory
experimental studies comparing online and offline programming performance.
_Communications of the ACM_, 11(1), 3–11.
[https://doi.org/10.1145/362851.362858](https://doi.org/10.1145/362851.362858)  
The earliest published large-scale measurement study of programmer productivity
variance. Observed factor-10 differences between best and worst performers.
Motivates the use of a 6:1 productivity ratio (principal vs. junior) in the HEE
tier model, which is a conservative calibration relative to this empirical range.

---

## Appendix A: Reference Baseline Parameter Set

```json
{
  "model_version": "1.0",
  "components": {
    "token_divisor":              500,
    "duration_seconds_divisor":     2,
    "step_weight":                  2,
    "tool_weight":                  3,
    "artifact_weight":            1.5,
    "artifact_cap":                20,
    "output_default_weight":        1,
    "review_weight":                4
  },
  "category_multipliers": {
    "uncategorized":              1.0,
    "summarization":              0.9,
    "drafting":                   1.0,
    "code_generation":            1.2,
    "root_cause_analysis":        1.4,
    "planning_architecture":      1.5,
    "regulated_decision_support": 1.6
  },
  "category_output_weights": {
    "uncategorized":              1.0,
    "summarization":              0.8,
    "drafting":                   1.0,
    "code_generation":            1.2,
    "root_cause_analysis":        1.0,
    "planning_architecture":      1.4,
    "regulated_decision_support": 1.4
  },
  "category_review_weights": {
    "uncategorized":               4,
    "summarization":               2,
    "drafting":                    3,
    "code_generation":             5,
    "root_cause_analysis":         6,
    "planning_architecture":       8,
    "regulated_decision_support": 10
  },
  "context_multipliers": {
    "retry_per_unit":              0.25,
    "retry_cap":                   2.0,
    "ambiguity_high_temp":         1.1,
    "ambiguity_temp_threshold":    0.7,
    "risk_high":                   1.2,
    "risk_medium":                 1.0,
    "risk_low":                    1.0,
    "human_in_loop":               0.7,
    "outcome": {
      "success":         1.0,
      "partial_success": 0.6,
      "failure":         0.25,
      "timeout":         0.2
    }
  },
  "confidence": {
    "base":                       0.95,
    "missing_category_penalty":   0.10,
    "missing_tokens_penalty":     0.15,
    "missing_steps_penalty":      0.05,
    "missing_tools_penalty":      0.05,
    "retry_penalty_per_unit":     0.05,
    "retry_penalty_cap":          0.20,
    "high_risk_cap":              0.85,
    "min":                        0.0,
    "max":                        1.0
  },
  "rounding": {
    "minutes_dp":    2,
    "hours_dp":      2,
    "days_dp":       2,
    "fec_dp":        2,
    "confidence_dp": 4
  }
}
```

---

## Appendix B: Notation Summary

| Symbol | Meaning |
|---|---|
| BCU | Base Cognitive Unit |
| $C_T, C_D, C_W, C_U, C_A, C_P, C_R$ | BCU components: tokens, duration, steps, tools, artifacts, output, review |
| $\delta_T, \delta_D$ | Divisors for token and duration components |
| $w_W, w_U, w_A$ | Weights for steps, tool calls, artifacts |
| $\omega_{P,\kappa}$ | Per-category output weight |
| $\rho_\kappa$ | Per-category review weight |
| $\mu_\kappa$ | Category multiplier |
| $\Gamma$ | Contextual multiplier product |
| $\gamma_r, \gamma_a, \gamma_k, \gamma_h, \gamma_o$ | Context sub-multipliers |
| $H_0$ | Baseline human minutes |
| $\pi_t$ | Tier productivity multiplier |
| $H_t$ | Tier-adjusted human minutes |
| $\lambda_t$ | Tier hourly rate |
| FEC | Financial Equivalent Cost |
| $\beta$ | Base confidence value |
| $c$ | Computed confidence score |
| $\kappa$ | Task category |
