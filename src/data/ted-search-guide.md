# EU TED Search API — Most Useful Searchable Fields

Compiled and cross-checked against the official EU Publications Office documentation:

- TED Developer Docs — [List of search fields](https://docs.ted.europa.eu/ODS/latest/reuse/field-list.html) (the canonical field list, ~1,000+ fields)
- TED Help — [Search & browse / Expert search syntax](https://ted.europa.eu/en/help/search-browse)
- eForms SDK — [Codelists reference](https://docs.ted.europa.eu/eforms/latest/reference/code-lists/index.html)
- EU Publications Office, TED reusers workshop Q&A (confirms the live endpoint)

The full field list contains one search field per eForms Business Term (well over 1,000 entries — mostly narrow fields like `organisation-email-mediator-lot`). Below is a curated set of the fields you'll actually need for most searches, organised by topic, each with its confirmed value format and a real example from the official docs.

---

## 1. API basics

- **Endpoint:** `POST https://api.ted.europa.eu/v3/notices/search` — anonymous/keyless for search & retrieval.
- **Swagger/OpenAPI reference:** https://ted.europa.eu/api/documentation/index.html
- **Request body fields:** `query`, `fields` (array of field names to return), `page`, `limit`, `scope`, `checkQuerySyntax`, `paginationMode`, `iterationNextToken`
- **`scope` values:** `ACTIVE` (open/live notices), `ALL` (~10-year archive), `LATEST` (latest OJ S issue)
- **`paginationMode`:** `PAGE_NUMBER` (max 15,000 notices total, 250/page) or `ITERATION` (scroll token, no cap on total results, 250/page)
- **Field name formats:** query field names are the current kebab-case "eForms search field" names (e.g. `publication-date`). Most also have a legacy 1–3 letter alias from the pre-2023 TED site (e.g. `PD`) that the official docs say resolves to the same field. In practice, a few third-party wrappers report inconsistent results with the legacy short codes — if one doesn't return what you expect, fall back to the full kebab-case name.

Minimal example request body:
```json
{
  "query": "place-of-performance IN (LUX)",
  "fields": ["publication-number", "notice-title", "buyer-name"],
  "limit": 50,
  "scope": "ACTIVE",
  "checkQuerySyntax": false,
  "paginationMode": "ITERATION"
}
```

---

## 2. Query syntax cheat-sheet

| Operator | Meaning | Example |
|---|---|---|
| `=` | Exact match (word/phrase/wildcard); multiple values need explicit `OR`/`AND` in parentheses | `FT = (agriculture OR food)` |
| `~` | Match with implicit `AND`; on multilingual fields the terms are **stemmed** | `FT ~ (agriculture food)` |
| `IN` | Match any value in a list (implicit `OR`) | `notice-type IN (cn-desg can-modif)` |
| `NOT IN` | Exclude any value in a list | `notice-type NOT IN (cn-desg veat)` |
| `!=` | Negated exact match | `winner-selection-status != *` (field is empty) |
| `>` `<` `>=` `<=` | Comparison — numeric, date, publication-number, OJ S number fields only | `PD >= 20260101` |
| `AND` `OR` `NOT` | Boolean combination, with parentheses for grouping | `FT ~ Diesel AND notice-type = can-standard` |
| `SORT BY` | Sort results (keyword/numeric fields only) | `SORT BY publication-date DESC` |

Other notes confirmed in the docs:
- **Wildcards:** `*` matches any number of characters (`classification-cpv = 30*`); not usable inside quoted phrases or as a leading wildcard when `scope=ALL`.
- **Exact phrases:** use double quotes, e.g. `buyer-name ~ "Council of the"`.
- **Date ranges:** `PD = (20240101<>20240131)`.
- **"Field exists" / "field empty":** `legal-basis-proc = *` (has a value) / `legal-basis-proc != *` (no value).
- **Time-period shorthand** (for duration-type fields): a number + `d`/`w`/`m`/`y`, e.g. `contract-duration-period-lot = 5d`.

---

## 3. Most useful fields

### Full text & notice identity

| Field | Alias | Filters | Value format & example |
|---|---|---|---|
| `FT` | — | Free-text search across all notice text and code labels, in every language | word / phrase / wildcard. `FT ~ (electric bus)` |
| `publication-number` | `ND` | The unique ID of a published notice | `NNNNNN-YYYY`. `ND = 291298-2024` |
| `notice-title` | `TI` | Notice title (multilingual) | free text/phrase. `TI ~ "road maintenance"` |
| `ojs-number` | `OJ` | OJ S (Official Journal Supplement) issue number | `NNN/YYYY`. `OJ = 123/2024` |

### Dates

| Field | Alias | Filters | Value format & example |
|---|---|---|---|
| `publication-date` | `PD` | Date the notice appeared on TED | `YYYYMMDD`; supports ranges and comparisons. `PD >= 20260101`, `PD = (20240101<>20240131)` |
| `dispatch-date` | `DS` | Date the notice was sent to the Publications Office | `YYYYMMDD`. `DS = 20240916` |
| `deadline-receipt-tender-date-lot` | — | Deadline for submitting tenders, per lot (BT-131) | `YYYYMMDD` |
| `deadline-receipt-request-date-lot` | — | Deadline for requests to participate, per lot (BT-1311, restricted-type procedures) | `YYYYMMDD` |
| `deadline-receipt-expressions-date-lot` | — | Deadline for expressions of interest, per lot (BT-630) | `YYYYMMDD` |
| `deadline-receipt-request` | `DT` | Legacy combined alias over the three submission deadlines above | `YYYYMMDD` |

### Classification & value

| Field | Alias | Filters | Value format & example |
|---|---|---|---|
| `classification-cpv` | `PC` | Main/additional CPV (Common Procurement Vocabulary) product or service code | 8-digit numeric code, wildcard allowed. `PC = 45*`; `classification-cpv IN (31642200 38000000 73410000)` |
| `contract-nature` | `NC` | High-level nature of the contract | one of `works`, `supplies`, `services`. `NC = supplies` |
| `estimated-value-proc` / `estimated-value-cur-proc` | — | Estimated total value of the procedure, and its currency | number / ISO 4217 code. `estimated-value-proc >= 500000 AND estimated-value-cur-proc = EUR` |
| `total-value` / `total-value-cur` | `TV` / `TV_CUR` | Broad "any value" match — spans estimated value, awarded value, framework max value etc. across many underlying terms; good for a quick threshold search, less precise than the specific `*-value-*` fields | number / ISO 4217 code. `TV = (>=1000000) AND TV_CUR = EUR` |

### Buyer / contracting authority

| Field | Alias | Filters | Value format & example |
|---|---|---|---|
| `buyer-name` | `AU` | Name of the contracting authority | free text (multilingual). `AU ~ "Ministry of Health"` |
| `buyer-country` | `CY` | Country of the buyer | 3-letter code, EU "country" list (ISO 3166 + extra authority codes). `CY = DEU` |
| `buyer-city` | `TW` | Town/city of the buyer | free text (multilingual) |
| `buyer-legal-type` | `AA` | Type of contracting authority (central government, regional authority, body governed by public law, EU institution, etc.) | code from the `buyer-legal-type` codelist |
| `main-activity` | `MA` | Main sector of activity of the buyer (health, education, defence, etc.) | code from the `main-activity` codelist |
| `buyer-identifier` | `BI` | Buyer's national registration number, or a short code for EU institutions | free text, or a small set of fixed codes. `buyer-identifier IN (CURIA PUBL)` |

### Procedure & legal basis

| Field | Alias | Filters | Value format & example |
|---|---|---|---|
| `procedure-type` | `PR` | Type of procurement procedure | code: `open`, `restricted`, `comp-tend`, `comp-dial`, `neg-w-call`, `neg-wo-call`, `innovation`, `oth-single`, `oth-mult`. `PR = open` |
| `legal-basis` | `DI` | EU directive/regulation the notice is issued under | code, usually a CELEX number, e.g. `32014L0024` (classic public sector), `32014L0025` (utilities), `32014L0023` (concessions) |
| `notice-type` | — | Specific notice subtype | code, e.g. `cn-standard`, `cn-social`, `cn-desg`, `pin-cfc-standard`, `pin-cfc-social`, `qu-sy`, `subco`, `can-standard`, `can-social`, `can-desg`, `veat`, `can-modif`. `notice-type IN (cn-standard cn-social)` |
| `form-type` | — | Broad notice family | code: `planning`, `competition`, `result`, `change`, `cont-modif`, `dir-awa-pre`, `bri` |

### Place of performance & language

| Field | Alias | Filters | Value format & example |
|---|---|---|---|
| `place-of-performance` | `RC` | Where the works/supplies/services will be delivered | NUTS code, or 3-letter country code, or a special value: `anyw` (anywhere), `anyw-eea` (anywhere in EEA), `00` (other/unspecified). `RC = FRF11`; `place-of-performance IN (GRC BEL)` |
| `official-language` | `OL` | Official EU language the notice is published in | 3-letter code, one of the 24 EU official languages. `OL = ENG` |
| `submission-language` | `SUB_LG` | Language(s) tenders may be submitted in | 3-letter EU language code |

### Notice status & contract award

| Field | Alias | Filters | Value format & example |
|---|---|---|---|
| `winner-selection-status` | — | Whether a winner has been chosen for the lot | code: `selec-w` (winner chosen), `open-nw` (competition open, no winner yet), `clos-nw` (closed, no winner). `winner-selection-status IN (selec-w)` |
| `winner-name` / `winner-country` | — | Name / country of the awarded economic operator | free text (multilingual) / 3-letter country code |
| `NL` | — | Number of lots in the procedure | integer, comparison operators. `NL >= 10` |

---

## 4. Coded fields — where the full value lists live

Several fields above only accept values from a fixed EU codelist. The ones you'll hit most, with confirmed values:

- **`procedure-type`** — `open`, `restricted`, `comp-tend`, `comp-dial`, `neg-w-call`, `neg-wo-call`, `innovation`, `oth-single`, `oth-mult` (plus `exp-int-rail` for rail-specific notices)
- **`contract-nature`** — `works`, `supplies`, `services`
- **`form-type`** — `planning`, `competition`, `result`, `change`, `cont-modif`, `dir-awa-pre`, `bri`
- **`winner-selection-status`** — `selec-w`, `open-nw`, `clos-nw`
- **`notice-type`** — dozens of codes, one set per form type (e.g. the `competition` form type allows `cn-standard`, `cn-social`, `cn-desg`, `pin-cfc-standard`, `pin-cfc-social`, `qu-sy`, `subco`)
- **Country fields** (`buyer-country`, `place-of-performance`, `winner-country`, etc.) — 3-letter codes correlated with ISO 3166-1, plus a few extra authority codes for entities without an ISO code
- **Language fields** (`official-language`, `submission-language`) — 3-letter codes, a subset of 24 EU official languages
- **Currency fields** (`total-value-cur`, `estimated-value-cur-proc`, etc.) — ISO 4217 codes (`EUR`, `USD`, ...)

Full, authoritative codelists (with all values in all 24 languages): https://docs.ted.europa.eu/eforms/latest/reference/code-lists/index.html

---

## 5. Example queries (from official documentation)

```
# Notices published in a date range
publication-date = (20240101<>20240131)

# Open procedures for construction work above €1M, in EUR
classification-cpv = 45* AND procedure-type = open AND total-value = (>=1000000) AND total-value-cur = EUR

# Contract notices or design contest notices, in Greece or Belgium
notice-type IN (cn-standard cn-desg) AND place-of-performance IN (GRC BEL)

# Full-text search restricted to standard contract award notices
FT ~ Diesel AND notice-type = can-standard

# Notices with more than 10 lots
NL >= 10

# Award notices published on a given date where a winner was chosen
notice-type IN (can-standard can-social can-desg can-tran) AND publication-date = 20240125 AND winner-selection-status IN (selec-w)
```

---

## 6. Full field list

This curated set covers the fields most searches need. For the complete list (organisation contact fields, review-body fields, sustainability/CVD/EED fields, framework-agreement fields, etc.), see the official downloadable versions:

- [HTML](https://docs.ted.europa.eu/ODS/latest/reuse/field-list.html) · [PDF](https://docs.ted.europa.eu/ODS/latest/reuse/_attachments/List_of_search_fields.pdf) · [XLSX](https://docs.ted.europa.eu/ODS/latest/reuse/_attachments/List_of_search_fields.xlsx) · [CSV](https://docs.ted.europa.eu/ODS/latest/reuse/_attachments/List_of_search_fields.csv)