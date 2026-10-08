
use serde_json::{Value};

use crate::utils::{ToolError};

use crate::tools::edgar::functions::filter_frame_data;



#[cfg(test)]
mod xbrl_frame_filter_tests {
    use super::*;
    use serde_json::json;

    fn sample_frame() -> Value {
        json!({
            "label": "Revenue",
            "description": "Revenue concept",
            "entityName": "SEC XBRL Frame",
            "data": [
                {
                    "accn": "0000913760-26-000017",
                    "cik": 913760,
                    "end": "2025-12-31",
                    "entityName": "StoneX Group Inc.",
                    "loc": "US-NY",
                    "start": "2025-10-01",
                    "val": 39029900000_i64
                },
                {
                    "accn": "0001045810-26-000010",
                    "cik": 1045810,
                    "end": "2025-12-31",
                    "entityName": "NVIDIA Corporation",
                    "loc": "US-CA",
                    "start": "2025-10-01",
                    "val": 68000000000_i64
                },
                {
                    "accn": "0000123456-26-000001",
                    "cik": 123456,
                    "end": "2025-12-31",
                    "entityName": "Example Small Company",
                    "loc": "US-NY",
                    "start": "2025-10-01",
                    "val": 500000_i64
                }
            ]
        })
    }

    fn apply_filter(arguments: Value) -> Result<Value, ToolError> {
        let mut data = sample_frame();
        filter_frame_data(&mut data, &arguments)?;
        Ok(data)
    }

    fn records(data: &Value) -> &[Value] {
        data["data"].as_array().unwrap()
    }

    fn ciks(data: &Value) -> Vec<u64> {
        records(data)
            .iter()
            .map(|record| record["cik"].as_u64().unwrap())
            .collect()
    }

    // 1. No optional filters: all records remain.
    #[test]
    fn test_no_filters_preserves_all_records() {
        let data = apply_filter(json!({})).unwrap();

        assert_eq!(records(&data).len(), 3);
        assert_eq!(ciks(&data), vec![913760, 1045810, 123456]);
    }

    // 2. CIK: exact match.
    #[test]
    fn test_cik_filter() {
        let data = apply_filter(json!({
            "cik": 913760
        }))
        .unwrap();

        assert_eq!(records(&data).len(), 1);
        assert_eq!(records(&data)[0]["entityName"], "StoneX Group Inc.");
    }

    // 3. Accession number: exact match.
    #[test]
    fn test_accn_filter() {
        let data = apply_filter(json!({
            "accn": "0001045810-26-000010"
        }))
        .unwrap();

        assert_eq!(records(&data).len(), 1);
        assert_eq!(records(&data)[0]["cik"], 1045810);
    }

    // 4. Entity name: case-insensitive substring match.
    #[test]
    fn test_entity_name_filter_is_case_insensitive() {
        let data = apply_filter(json!({
            "entity_name": "nViDiA"
        }))
        .unwrap();

        assert_eq!(records(&data).len(), 1);
        assert_eq!(records(&data)[0]["cik"], 1045810);
    }

    // 5. Location: exact match.
    #[test]
    fn test_loc_filter() {
        let data = apply_filter(json!({
            "loc": "US-NY"
        }))
        .unwrap();

        assert_eq!(records(&data).len(), 2);

        assert_eq!(
            ciks(&data),
            vec![913760, 123456]
        );
    }

    // 6. '+' means greater than or equal to the threshold.
    #[test]
    fn test_val_filter_greater_than_or_equal() {
        let data = apply_filter(json!({
            "val": "39029900000+"
        }))
        .unwrap();

        assert_eq!(records(&data).len(), 2);
        assert_eq!(
            ciks(&data),
            vec![913760, 1045810]
        );
    }

    // 7. '-' means less than or equal to the threshold.
    #[test]
    fn test_val_filter_less_than_or_equal() {
        let data = apply_filter(json!({
            "val": "500000-"
        }))
        .unwrap();

        assert_eq!(records(&data).len(), 1);
        assert_eq!(records(&data)[0]["cik"], 123456);
    }

    // 8. Combined filters use AND semantics.
    #[test]
    fn test_combined_filters() {
        let data = apply_filter(json!({
            "loc": "US-NY",
            "entity_name": "stonex",
            "val": "1000000000+"
        }))
        .unwrap();

        assert_eq!(records(&data).len(), 1);
        assert_eq!(records(&data)[0]["cik"], 913760);
    }

    // 9. A valid filter with no matches returns an empty array.
    #[test]
    fn test_filter_with_no_matches() {
        let data = apply_filter(json!({
            "cik": 999999999
        }))
        .unwrap();

        assert!(records(&data).is_empty());
    }

    // 10. Exact-match syntax for val is rejected.
    #[test]
    fn test_val_filter_rejects_exact_match_syntax() {
        let result = apply_filter(json!({
            "val": "1000000"
        }));

        assert!(matches!(result, Err(ToolError::InvalidInput(_))));
    }

    // 11. Invalid numeric thresholds are rejected.
    #[test]
    fn test_val_filter_rejects_invalid_number() {
        let result = apply_filter(json!({
            "val": "abc+"
        }));

        assert!(matches!(result, Err(ToolError::InvalidInput(_))));
    }

    // 12. Non-finite thresholds are rejected.
    #[test]
    fn test_val_filter_rejects_non_finite_number() {
        let result = apply_filter(json!({
            "val": "NaN+"
        }));

        assert!(matches!(result, Err(ToolError::InvalidInput(_))));
    }

    // 13. Missing or non-numeric record values do not match a threshold.
    #[test]
    fn test_val_filter_excludes_records_without_numeric_values() {
        let mut data = json!({
            "data": [
                {"cik": 1, "val": 100},
                {"cik": 2, "val": null},
                {"cik": 3}
            ]
        });

        filter_frame_data(
            &mut data,
            &json!({"val": "50+"}),
        )
        .unwrap();

        assert_eq!(records(&data).len(), 1);
        assert_eq!(records(&data)[0]["cik"], 1);
    }
}