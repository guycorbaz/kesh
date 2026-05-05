# Fixtures CAMT.053 / CSV — provenance

Tous les fichiers de ce dossier sont **synthétiques**, construits à la main
pour les tests unitaires des parseurs `kesh-import`. Ils ne contiennent
aucune donnée personnelle ni transactionnelle réelle :

- Les IBAN suisses utilisés sont des **IBAN fictifs MOD-97 valides** issus de
  la documentation publique des banques (PostFinance, UBS) ou des comptes de
  test ouverts publiquement. Aucun de ces IBAN ne correspond à un compte de
  production.
- Les noms de contreparties (« Acme SA », « Régie Genevoise SA », etc.) sont
  inventés.
- Les références ESR/SCOR sont des chaînes valides du point de vue du format
  ISO 11649 ou du standard QR-Référence suisse mais ne correspondent à
  aucun bulletin de versement réel.
- Les montants et dates sont arbitraires.

## Schémas de référence

- CAMT.053 v04 : ISO 20022 message standard (`urn:iso:std:iso:20022:tech:xsd:camt.053.001.04`).
- CAMT.053 v08 : SIX cash-management schemas v2.0.2 (`docs/six-references/ig-cash-managment-xml-schemas-v2.0.2-en/camt.053.001.08.xsd`).

## Catalogue (Story 8-1a)

Les 10 fichiers `camt053/*.xml` couvrent les variations attendues par le
parseur (AC #5, #9 de la story 8-1a) :

| Fichier | Cas couvert |
|---|---|
| `v04_minimal.xml` | namespace par défaut, 1 stmt, 1 Ntry sans TxDtls |
| `v04_prefixed_namespace.xml` | namespace préfixé `<ns:Document xmlns:ns="...">` (régression H4) |
| `v08_minimal.xml` | namespace v08 avec dispatcher |
| `v04_with_subtxs.xml` | 1 Ntry agrégée + 3 TxDtls → 3 transactions distinctes (FR49) |
| `v04_multi_stmt.xml` | 2 Stmt avec IBAN différents → 2 ImportedStatement |
| `v04_balance_mismatch.xml` | écart `\|opening + Σ - closing\| > 0.01` (CR-010 #62) |
| `v04_truncated.xml` | XML coupé en plein milieu — `MalformedXml` attendu |
| `v04_invalid_iban.xml` | IBAN counterparty checksum cassé conservé brut (§iban-tolerant) |
| `v04_eur_currency.xml` | `<Acct><Ccy>EUR</Ccy>` — devise extraite, rejet côté `kesh-core` |
| `v04_credit_debit_indicator.xml` | 1 DBIT + 1 CRDT → signe appliqué au montant |
