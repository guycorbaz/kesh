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

Les 14 fichiers `camt053/*.xml` (recompté par `ls camt053/ | wc -l`, Story 27-2) couvrent les variations attendues par le
parseur (AC #5, #9 de la story 8-1a) :

| Fichier | Cas couvert |
|---|---|
| `v04_minimal.xml` | namespace par défaut, 1 stmt, 1 Ntry à 1 TxDtls |
| `v04_prefixed_namespace.xml` | namespace préfixé `<ns:Document xmlns:ns="...">` (régression H4) |
| `v08_minimal.xml` | namespace v08 avec dispatcher |
| `v04_with_subtxs.xml` | 1 Ntry agrégée + 3 TxDtls → 3 transactions distinctes (FR49) |
| `v04_multi_stmt.xml` | 2 Stmt avec IBAN différents → 2 ImportedStatement |
| `v04_balance_mismatch.xml` | écart `\|opening + Σ - closing\| > 0.01` (CR-010 #62) |
| `v04_truncated.xml` | XML coupé en plein milieu — `MalformedXml` attendu |
| `v04_invalid_iban.xml` | IBAN counterparty checksum cassé conservé brut (§iban-tolerant) |
| `v04_eur_currency.xml` | `<Acct><Ccy>EUR</Ccy>` — devise extraite, rejet côté `kesh-core` |
| `v04_credit_debit_indicator.xml` | 1 DBIT + 1 CRDT → signe appliqué au montant |
| `v04_ntry_sign_fallback.xml` | `Ntry` sans `CdtDbtInd`, 2 TxDtls signés → signe repris de chaque TxDtls |
| `v04_overlap.xml` | `MsgId` distinct de `v04_minimal.xml`, une transaction identique en clé composite → doublon de ligne, pas de fichier (Story 8-3) |
| `v04_truncated_after_stmt_close.xml` | XML coupé après la fermeture d'un `Stmt` — `MalformedXml` par la garde de fin de fichier |
| `v04_amtdtls_foreign_currency.xml` | compte CHF, montants d'information en EUR (`AmtDtls/InstdAmt`, `Chrgs`) sur 3 entrées (TxDtls à `Amt` direct, sans TxDtls, TxDtls sans `Amt`) → chaque transaction en CHF (Story 27-2, AC19) |

## CSV (Story 26-1)

`csv/ubs_synthetique.csv` reproduit **la structure** du relevé CSV que
produit l'e-banking d'UBS, sans aucune de ses valeurs : le fichier est
**inventé de toutes pièces** (le relevé réel qui a servi à relever la
structure n'est jamais copié dans le dépôt, ni en entier, ni en extrait).

Structure reproduite :

- octets `EF BB BF` (BOM UTF-8) en tête, fins de ligne **LF**, **aucun LF
  après la dernière ligne** ;
- lignes 1 à 8 : préambule clé/valeur à 3 champs (`Clé:;valeur;`) ;
- ligne 9 : **vide** (aucun caractère) — elle ne compte pas dans les
  « lignes à ignorer », d'où le réglage **9** et non 10 ;
- ligne 10 : 14 titres et un `;` final (15 champs) ;
- lignes 11 à 21 : 11 transactions, dates **décroissantes**, débits
  (colonne 5) écrits **avec un signe moins**, crédits (colonne 6) positifs,
  jamais les deux sur une ligne ; descriptions parfois entre guillemets et
  contenant des `;`.

Invariants (re-vérifiés par les tests, pas à croire) : 11 transactions,
9 sorties et 2 entrées ; somme des montants signés **−236.25** = solde
final (4763.75) − solde initial (5000.00) du préambule ; la colonne Solde
est le solde courant après chaque ligne. Les paiements par carte ont une
date de transaction (colonne 0) antérieure à la date de comptabilisation
(colonne 2), ce qui discrimine le mapping recommandé `date = 2`. L'IBAN est
l'exemple public des documentations bancaires suisses
(`CH9300762011623852957`), aucun compte réel ; noms, montants et numéros
de transaction sont fictifs.
