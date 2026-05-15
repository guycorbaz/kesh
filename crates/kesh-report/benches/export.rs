//! Benchmark criterion T4.3 — Performance des sérialiseurs PDF/CSV (Story 9-2a).
//!
//! Mesure les ACs :
//! - **AC #9** : rendering PDF < 500 ms sur dataset 1000 écritures.
//! - **AC #30** : rendering CSV < 5 s sur dataset 10'000 écritures.
//! - **AC #31** : taille PDF < 5 MB sur dataset 10'000 écritures (via groupe `pdf_size_check`).
//!
//! Fixtures construites en code Rust pur (factories `make_*(n)`) — pas de DB
//! (Pass 1 AA-M4 + Q6 — isolation des effets DB latence/buffer pool).
//!
//! Exécution locale :
//! ```sh
//! cargo bench -p kesh-report --bench export
//! ```

use chrono::NaiveDate;
use criterion::{Criterion, criterion_group, criterion_main};
use kesh_db::entities::AccountType;
use kesh_db::entities::journal_entry::Journal;
use kesh_report::balance_sheet::{AccountBalance, BalanceSheet};
use kesh_report::income_statement::IncomeStatement;
use kesh_report::journal_report::{
    JournalEntryLineRow, JournalEntryRow, JournalReport, JournalSection,
};
use kesh_report::pdf::PdfContext;
use kesh_report::period::ReportPeriod;
use kesh_report::trial_balance::{TrialBalance, TrialBalanceRow};
use kesh_report::{
    render_balance_sheet_csv, render_balance_sheet_pdf, render_income_statement_csv,
    render_income_statement_pdf, render_journal_report_csv, render_journal_report_pdf,
    render_trial_balance_csv, render_trial_balance_pdf,
};
use rust_decimal::Decimal;
use std::str::FromStr;

fn period() -> ReportPeriod {
    ReportPeriod {
        fiscal_year_id: 1,
        start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    }
}

fn make_account_balance(i: usize, ty: AccountType) -> AccountBalance {
    AccountBalance {
        account_id: i as i64,
        account_number: format!("{:04}", 1000 + i),
        account_name: format!("Compte test #{i}"),
        account_type: ty,
        active: true,
        balance: Decimal::from_str(&format!("{}.{:02}", i, (i * 17) % 100)).unwrap(),
    }
}

fn make_balance_sheet(n: usize) -> BalanceSheet {
    let half = n / 2;
    let assets: Vec<AccountBalance> = (0..half)
        .map(|i| make_account_balance(i, AccountType::Asset))
        .collect();
    let liabilities: Vec<AccountBalance> = (half..n)
        .map(|i| make_account_balance(i, AccountType::Liability))
        .collect();
    let total_assets: Decimal = assets.iter().map(|a| a.balance).sum();
    let total_liabilities: Decimal = liabilities.iter().map(|a| a.balance).sum();
    BalanceSheet {
        period: period(),
        assets,
        liabilities,
        total_assets,
        total_liabilities,
        equity_result: total_assets - total_liabilities,
        equation_holds: true,
    }
}

fn make_income_statement(n: usize) -> IncomeStatement {
    let half = n / 2;
    let revenues: Vec<AccountBalance> = (0..half)
        .map(|i| make_account_balance(i, AccountType::Revenue))
        .collect();
    let expenses: Vec<AccountBalance> = (half..n)
        .map(|i| make_account_balance(i, AccountType::Expense))
        .collect();
    let total_revenues: Decimal = revenues.iter().map(|a| a.balance).sum();
    let total_expenses: Decimal = expenses.iter().map(|a| a.balance).sum();
    IncomeStatement {
        period: period(),
        revenues,
        expenses,
        total_revenues,
        total_expenses,
        net_result: total_revenues - total_expenses,
    }
}

fn make_trial_balance(n: usize) -> TrialBalance {
    let rows: Vec<TrialBalanceRow> = (0..n)
        .map(|i| TrialBalanceRow {
            account_id: i as i64,
            account_number: format!("{:04}", 1000 + i),
            account_name: format!("Compte #{i}"),
            account_type: if i % 2 == 0 {
                AccountType::Asset
            } else {
                AccountType::Liability
            },
            active: true,
            total_debit: Decimal::from(100),
            total_credit: Decimal::from(100),
            balance: Decimal::from(0),
        })
        .collect();
    let total_debit: Decimal = rows.iter().map(|r| r.total_debit).sum();
    let total_credit: Decimal = rows.iter().map(|r| r.total_credit).sum();
    TrialBalance {
        period: period(),
        rows,
        total_debit,
        total_credit,
        balanced: true,
    }
}

fn make_journal_report(n_entries: usize) -> JournalReport {
    // Réparti sur 5 journaux.
    let journals = [
        Journal::Achats,
        Journal::Ventes,
        Journal::Banque,
        Journal::Caisse,
        Journal::OD,
    ];
    let per_section = n_entries / 5;
    let mut sections = Vec::with_capacity(5);
    let mut grand_debit = Decimal::ZERO;
    let mut grand_credit = Decimal::ZERO;
    for (idx, j) in journals.iter().enumerate() {
        let entries: Vec<JournalEntryRow> = (0..per_section)
            .map(|i| JournalEntryRow {
                entry_id: (idx * per_section + i) as i64,
                entry_number: i as i64,
                entry_date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
                description: format!("Écriture test #{i}"),
                lines: vec![
                    JournalEntryLineRow {
                        account_id: 1,
                        account_number: "1000".into(),
                        account_name: "Caisse".into(),
                        debit: Decimal::from(100),
                        credit: Decimal::ZERO,
                        line_order: 0,
                    },
                    JournalEntryLineRow {
                        account_id: 2,
                        account_number: "2000".into(),
                        account_name: "Fournisseurs".into(),
                        debit: Decimal::ZERO,
                        credit: Decimal::from(100),
                        line_order: 1,
                    },
                ],
            })
            .collect();
        let section_debit = Decimal::from((per_section as i64) * 100);
        let section_credit = Decimal::from((per_section as i64) * 100);
        grand_debit += section_debit;
        grand_credit += section_credit;
        sections.push(JournalSection {
            journal: *j,
            entries,
            section_total_debit: section_debit,
            section_total_credit: section_credit,
        });
    }
    JournalReport {
        period: period(),
        journals: sections,
        grand_total_debit: grand_debit,
        grand_total_credit: grand_credit,
    }
}

fn ctx() -> PdfContext {
    PdfContext::fr_ch_default("CI Test Company")
}

fn bench_pdf_1000(c: &mut Criterion) {
    let bs = make_balance_sheet(1000);
    let is = make_income_statement(1000);
    let tb = make_trial_balance(1000);
    let jr = make_journal_report(1000);
    let ctx = ctx();

    let mut group = c.benchmark_group("pdf_1000");
    group.sample_size(10);
    group.bench_function("balance_sheet", |b| {
        b.iter(|| render_balance_sheet_pdf(&bs, &ctx).unwrap())
    });
    group.bench_function("income_statement", |b| {
        b.iter(|| render_income_statement_pdf(&is, &ctx).unwrap())
    });
    group.bench_function("trial_balance", |b| {
        b.iter(|| render_trial_balance_pdf(&tb, &ctx).unwrap())
    });
    group.bench_function("journal_report", |b| {
        b.iter(|| render_journal_report_pdf(&jr, &ctx).unwrap())
    });
    group.finish();
}

fn bench_csv_1000(c: &mut Criterion) {
    let bs = make_balance_sheet(1000);
    let is = make_income_statement(1000);
    let tb = make_trial_balance(1000);
    let jr = make_journal_report(1000);

    let mut group = c.benchmark_group("csv_1000");
    group.sample_size(10);
    group.bench_function("balance_sheet", |b| {
        b.iter(|| {
            let mut buf = Vec::new();
            render_balance_sheet_csv(&bs, &mut buf).unwrap();
            buf
        })
    });
    group.bench_function("income_statement", |b| {
        b.iter(|| {
            let mut buf = Vec::new();
            render_income_statement_csv(&is, &mut buf).unwrap();
            buf
        })
    });
    group.bench_function("trial_balance", |b| {
        b.iter(|| {
            let mut buf = Vec::new();
            render_trial_balance_csv(&tb, &mut buf).unwrap();
            buf
        })
    });
    group.bench_function("journal_report", |b| {
        b.iter(|| {
            let mut buf = Vec::new();
            render_journal_report_csv(&jr, &mut buf).unwrap();
            buf
        })
    });
    group.finish();
}

fn bench_pdf_10k(c: &mut Criterion) {
    let jr = make_journal_report(10_000);
    let ctx = ctx();
    let mut group = c.benchmark_group("pdf_10k");
    group.sample_size(10);
    group.bench_function("journal_report_pagination", |b| {
        b.iter(|| {
            let bytes = render_journal_report_pdf(&jr, &ctx).unwrap();
            // AC #31 — taille < 5 MB
            assert!(
                bytes.len() < 5 * 1024 * 1024,
                "PDF 10k entries must be < 5 MB, got {} bytes",
                bytes.len()
            );
            bytes
        })
    });
    group.finish();
}

fn bench_csv_10k(c: &mut Criterion) {
    let jr = make_journal_report(10_000);
    let mut group = c.benchmark_group("csv_10k");
    group.sample_size(10);
    group.bench_function("journal_report", |b| {
        b.iter(|| {
            let mut buf = Vec::new();
            render_journal_report_csv(&jr, &mut buf).unwrap();
            buf
        })
    });
    group.finish();
}

criterion_group!(benches, bench_pdf_1000, bench_csv_1000, bench_pdf_10k, bench_csv_10k);
criterion_main!(benches);
