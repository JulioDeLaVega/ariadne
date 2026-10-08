# Ariadne

**An MCP server for accessing structured data from public APIs and databases.**

Ariadne connects AI assistants to authoritative public data sources through the **Model Context Protocol (MCP)**.

It provides a unified interface for discovering, querying, and navigating structured public data. The architecture is designed to make it easy to add new APIs and databases over time.

## Current integrations

| Source | Data |
|---|---|
| **EU Publications Office / CELLAR** | EU legislation, documents, metadata and legal relationships |
| **TED** | EU public procurement notices and awards |
| **SEC EDGAR** | US public-company identification and XBRL financial data |

More public APIs and databases can be integrated using the same MCP architecture.

---

## What Ariadne does

Ariadne is designed for **structured research**, rather than simply retrieving webpages.

Depending on the source, Ariadne can:

- search structured datasets
- resolve identifiers
- navigate relationships between resources
- retrieve different representations of a resource
- extract targeted information from documents
- retrieve structured historical data

The tools are intentionally **composable**. An AI assistant can use the output of one tool as the input to another.

Example:

```text
Identifier
    ↓
Resource
    ↓
Related resources
    ↓
Specific representation
    ↓
Targeted information
```

---

# Integrations

## EU Publications Office / CELLAR

Ariadne exposes the CELLAR RDF graph and document repository through a set of MCP tools.

```text
cellar.get_works_based_on_keyword
cellar.uri_from_celex
cellar.get_predicates
cellar.get_objects_based_on_uri
cellar.get_resource_adopts_resource
cellar.get_resource_legal_information_miscellaneous
cellar.get_expressions_based_on_work
cellar.get_manifestations_based_on_expression
cellar.get_manifestation_content_with_regex
```

The integration covers:

- regulation discovery
- CELEX → CELLAR URI resolution
- RDF graph exploration
- reverse legal relationships
- adoption relationships
- legislative procedure information
- language expressions
- document manifestations and formats
- targeted document content extraction

A typical document-retrieval workflow is:

```text
CELEX
  ↓
CELLAR work
  ↓
Language expression
  ↓
Manifestation
  ↓
Document content
  ↓
Targeted extraction
```

The CELLAR implementation uses the EU Publications Office SPARQL endpoint and supports formats including XHTML, PDF/A and Formex XML.  

---

## TED

Ariadne integrates the official **TED Search API** for EU public procurement.

### Implemented

```text
ted.search_notices
ted.search_awards
```

The notice search covers structured fields such as:

- publication number and date
- notice type
- title
- procedure description
- CPV classification
- buyer
- country
- place of performance
- estimated value
- tender deadline
- procedure identifier

The award search additionally exposes winner and contract information, including winner identity, country, value and contract conclusion date.

Ariadne also exposes the MCP resource:

```text
ted://search-guide
```

which contains guidance for constructing TED search queries.

---

# SEC EDGAR

Ariadne provides structured access to selected **SEC EDGAR APIs**.

The current implementation intentionally focuses on company identification and XBRL data. Filing and ownership APIs are not yet implemented.

## EDGAR API coverage

```text
EDGAR
│
├── Company identification
│   └── Company Tickers          ✅
│
├── XBRL
│   ├── Company Facts             ✅
│   ├── Company Concept           ✅
│   └── Frames                    ✅
│
├── Company filings
│   ├── Submissions               ❌
│   ├── Filing search             ❌
│   ├── Filing archives           ❌
│   └── Filing indexes            ❌
│
└── Ownership
    ├── Forms 3/4/5               ❌
    ├── 13F                       ❌
    ├── 13D/13G                   ❌
    └── Form 144                  ❌
```

### Implemented tools

```text
edgar.get_cik
edgar.get_company_tags
edgar.get_company_concept
```

### `edgar.get_cik`

Resolves a US company ticker to its SEC Central Index Key (CIK).

```json
{
  "ticker": "NVDA"
}
```

The SEC company ticker database is loaded on first use and cached in memory for subsequent lookups. 

### `edgar.get_company_tags`

Discovers the XBRL concepts reported by a company.

```json
{
  "cik": "0001045810"
}
```

It returns a catalog of concepts across SEC taxonomies, including:

- taxonomy
- XBRL tag
- label
- available units
- number of facts
- earliest reporting period
- latest reporting period

This is intended as a discovery step before requesting a specific concept. 

### `edgar.get_company_concept`

Retrieves historical facts for a specific XBRL concept.

```json
{
  "cik": "0001045810",
  "taxonomy": "us-gaap",
  "tag": "RevenueFromContractWithCustomerExcludingAssessedTax"
}
```

The response includes the company, concept metadata, units and historical reported facts. Large historical responses are bounded before being returned. 

The intended workflow is:

```text
Ticker
  ↓
edgar.get_cik
  ↓
CIK
  ↓
edgar.get_company_tags
  ↓
Taxonomy + XBRL tag
  ↓
edgar.get_company_concept
  ↓
Historical facts
```

---

# Architecture

```text
                    MCP Client
               ┌──────────────────┐
               │ ChatGPT / Claude │
               │  Other MCP hosts │
               └────────┬─────────┘
                        │
                        │ MCP / HTTP
                        ▼
                 ┌───────────────┐
                 │    Ariadne    │
                 │   MCP Server  │
                 └───────┬───────┘
                         │
          ┌──────────────┼──────────────┐
          ▼              ▼              ▼
      CELLAR            TED         SEC EDGAR
      EU law        Procurement      Finance

                         ...
                         │
                  Future public APIs
                  and databases
```

Each integration encapsulates the source-specific API or query logic while exposing it through the same MCP server.

This allows Ariadne to grow into a broader gateway for structured public information without changing the interface used by MCP clients.

---

# Design principles

### Public and authoritative sources

Ariadne prioritizes APIs, databases and knowledge graphs operated by public institutions or primary data providers.

### Structured access

The goal is to expose structured records and relationships rather than treating every source as a collection of webpages.

### Composable tools

Tools should be useful individually but particularly powerful when chained together.

### Targeted retrieval

Large datasets and documents are queried, filtered or bounded where appropriate so that responses remain practical for AI clients.

### Extensible integrations

New public APIs should be addable without redesigning the MCP layer.

---

# Running locally

Build:

```bash
cargo build --release
```

Run:

```bash
cargo run --release
```

The server listens on:

```text
0.0.0.0:8080
```

The MCP endpoint is:

```text
POST /mcp
```

Example:

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/list",
    "params": {}
  }'
```

---

# Project status

Ariadne is under active development.

Current integrations cover:

- **EU legislation and legal metadata** through CELLAR
- **EU public procurement** through TED
- **US company identification and XBRL data** through SEC EDGAR

The project is intended to expand to additional **public APIs, databases and structured information sources** over time.

---

# License

Add the applicable license here.