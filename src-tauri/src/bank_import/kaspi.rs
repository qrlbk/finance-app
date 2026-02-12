//! Kaspi Bank statement parser
//!
//! Parses Kaspi Gold PDF statements and extracts transactions.

use super::{BankParser, ParsedStatement, ParsedTransaction};
use regex::Regex;

/// Parser for Kaspi Bank statements
pub struct KaspiParser;

impl BankParser for KaspiParser {
    fn bank_name(&self) -> &'static str {
        "Kaspi Bank"
    }

    fn can_parse(&self, text: &str) -> bool {
        text.contains("Kaspi Bank") && (text.contains("Kaspi Gold") || text.contains("ВЫПИСКА"))
    }

    fn parse(&self, text: &str) -> Result<ParsedStatement, String> {
        let (period_start, period_end) = parse_period(text)?;
        let account = parse_account(text);
        let card = parse_card(text);
        let transactions = parse_transactions(text)?;

        Ok(ParsedStatement {
            bank: self.bank_name().to_string(),
            period_start,
            period_end,
            account,
            card,
            transactions,
        })
    }
}

/// Parse the statement period from text
fn parse_period(text: &str) -> Result<(String, String), String> {
    // Pattern: "за период с DD.MM.YY по DD.MM.YY"
    let period_re =
        Regex::new(r"за период с (\d{2}\.\d{2}\.\d{2}) по (\d{2}\.\d{2}\.\d{2})").unwrap();

    if let Some(caps) = period_re.captures(text) {
        let start = convert_date(&caps[1])?;
        let end = convert_date(&caps[2])?;
        return Ok((start, end));
    }

    Err("Не удалось определить период выписки".to_string())
}

/// Parse account number
fn parse_account(text: &str) -> Option<String> {
    // Pattern: "Номер счета: KZ..."
    let account_re = Regex::new(r"Номер счета:\s*(KZ\w+)").unwrap();
    account_re
        .captures(text)
        .map(|caps| caps[1].to_string())
}

/// Parse card number (masked)
fn parse_card(text: &str) -> Option<String> {
    // Pattern: "Номер карты: *1234"
    let card_re = Regex::new(r"Номер карты:\s*\*?(\d+)").unwrap();
    card_re
        .captures(text)
        .map(|caps| format!("*{}", &caps[1]))
}

/// Convert date from DD.MM.YY to YYYY-MM-DD
fn convert_date(date_str: &str) -> Result<String, String> {
    let parts: Vec<&str> = date_str.split('.').collect();
    if parts.len() != 3 {
        return Err(format!("Неверный формат даты: {}", date_str));
    }

    let day = parts[0];
    let month = parts[1];
    let year = parts[2];

    // Convert 2-digit year to 4-digit (assume 20xx for years < 50, 19xx otherwise)
    let full_year = if year.len() == 2 {
        let y: i32 = year.parse().map_err(|_| "Неверный год")?;
        if y < 50 {
            format!("20{:02}", y)
        } else {
            format!("19{:02}", y)
        }
    } else {
        year.to_string()
    };

    Ok(format!("{}-{}-{}", full_year, month, day))
}

/// Parse all transactions from the statement text
fn parse_transactions(text: &str) -> Result<Vec<ParsedTransaction>, String> {
    let mut transactions = Vec::new();

    // Line-by-line approach - more reliable without look-ahead
    for line in text.lines() {
        if let Some(tx) = parse_transaction_line(line) {
            transactions.push(tx);
        }
    }

    // If line-by-line didn't work, try a simpler multiline pattern
    if transactions.is_empty() {
        // Simple pattern without look-ahead
        let tx_re = Regex::new(
            r"(\d{2}\.\d{2}\.\d{2})\s+([+-])\s*([\d\s]+,\d{2})\s*₸\s+(Покупка|Пополнение|Перевод|Снятие|Разное)\s+([^\n]+)"
        ).map_err(|e| format!("Regex error: {}", e))?;

        for caps in tx_re.captures_iter(text) {
            if let Some(tx) = parse_capture(&caps) {
                transactions.push(tx);
            }
        }
    }

    if transactions.is_empty() {
        return Err("Не найдено транзакций в выписке".to_string());
    }

    Ok(transactions)
}

/// Parse a single transaction line
fn parse_transaction_line(line: &str) -> Option<ParsedTransaction> {
    // Pattern: DD.MM.YY  +/- amount ₸  Type  Description
    // More flexible pattern to handle various whitespace
    let line_re = Regex::new(
        r"(\d{2}\.\d{2}\.\d{2})\s+([+-])\s*([\d\s]+,\d{2})\s*₸\s+(Покупка|Пополнение|Перевод|Снятие|Разное)\s+(.+)"
    ).ok()?;

    let caps = line_re.captures(line.trim())?;
    parse_capture(&caps)
}

/// Parse a regex capture into a transaction
fn parse_capture(caps: &regex::Captures) -> Option<ParsedTransaction> {
    let date_str = caps.get(1)?.as_str();
    let sign = caps.get(2)?.as_str();
    let amount_str = caps.get(3)?.as_str();
    let op_type = caps.get(4)?.as_str();
    let description = caps.get(5)?.as_str().trim();

    // Convert date
    let date = convert_date(date_str).ok()?;

    // Parse amount (remove spaces, replace comma with dot)
    let amount_clean = amount_str.replace(' ', "").replace(',', ".");
    let amount: f64 = amount_clean.parse().ok()?;

    // Determine transaction type based on sign and operation
    let transaction_type = match (sign, op_type) {
        ("+", _) => "income",
        ("-", _) => "expense",
        (_, "Пополнение") => "income",
        _ => "expense",
    }
    .to_string();

    // Clean description (remove extra whitespace, trim)
    let description = description
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    Some(ParsedTransaction {
        date,
        amount,
        transaction_type,
        description,
        original_type: op_type.to_string(),
        suggested_category_id: None,
        confidence: None,
        is_duplicate: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_date() {
        assert_eq!(convert_date("12.02.26").unwrap(), "2026-02-12");
        assert_eq!(convert_date("01.01.99").unwrap(), "1999-01-01");
        assert_eq!(convert_date("31.12.25").unwrap(), "2025-12-31");
    }

    #[test]
    fn test_parse_transaction_line() {
        let line = "12.02.26 - 1 385,00 ₸   Покупка      Южный";
        let tx = parse_transaction_line(line);
        assert!(tx.is_some());
        let tx = tx.unwrap();
        assert_eq!(tx.date, "2026-02-12");
        assert_eq!(tx.amount, 1385.0);
        assert_eq!(tx.transaction_type, "expense");
        assert_eq!(tx.original_type, "Покупка");
    }

    #[test]
    fn test_parse_income_line() {
        let line = "12.02.26 + 5 000,00 ₸   Пополнение      Асылбек Е.";
        let tx = parse_transaction_line(line);
        assert!(tx.is_some());
        let tx = tx.unwrap();
        assert_eq!(tx.amount, 5000.0);
        assert_eq!(tx.transaction_type, "income");
        assert_eq!(tx.original_type, "Пополнение");
    }

    #[test]
    fn test_kaspi_parser_can_parse() {
        let parser = KaspiParser;
        assert!(parser.can_parse("АО «Kaspi Bank», БИК CASPKZKA, www.kaspi.kz\nВЫПИСКА\nпо Kaspi Gold"));
        assert!(!parser.can_parse("Some other bank statement"));
    }
}
