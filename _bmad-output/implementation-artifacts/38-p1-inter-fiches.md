# Epic 38 — findings inter-fiches de la validation P1 (2026-11-15), à appliquer

Relevés par les cinq passes P1 (Sonnet) ; chaque fiche a déjà ses correctifs propres (commit `a9d6215d`).
Décisions de l'orchestrateur : **D-P1-1** découper 38-3 en **38-3a** (schéma non breaking, colonnes, entités,
décodage tolérant de `PROVIDER_API`, gardes : suppression d'import refusée, import de fichier refusé, soldes vus) et
**38-3b** (crate `kesh-providers`, trait `Connector`, transport, passage, planification, contrôle des soldes) — la
fiche 38-3 touche six modules (règle de splitting). **D-P1-2** : 38-5-P1-5 (compte d'acquisitions égal à d'autres
passifs) reste LOW, non traité.

## Epic 38 (`_bmad-output/planning-artifacts/epic-38-paypal.md`) et `sprint-status.yaml`

1. E4, E19, C-38-0 et la table des stories disent encore que la migration breaking est en 38-3 : la placer en **38-2**
   (O1). Le décodage tolérant `Other(String)` arrive avec la première écriture d'une valeur neuve (`PROVIDER_API` en
   38-3a, critères en 38-4) ; la migration de 38-2 ne change que le DDL.
2. E11 et l'inventaire : « compte `bank` sans IBAN admis depuis 34-11a1 » est faux (seule la carte en est dispensée).
3. E6 : « `rows_affected` distingue inséré et connu » → « lecture préalable des identifiants connus » (`CLIENT_FOUND_ROWS`).
4. E16 : conversion dont un côté est en CHF = écriture unique (C-38-6b-2 bis).
5. O10/O11 propagés : découpage de fenêtre dans le connecteur, `overlapDays` (1–30, défaut 3), `accountEmail` retiré.
6. Dépendances : 38-6a dépend de 38-3, 38-4, 38-5 et **38-6c** ; 38-6b de 38-2, 38-3, 38-6a, 38-6c.
7. C-38-8 : ajouter les codes `VAT_KIND_MISMATCH`, `VAT_KIND_UNKNOWN`, `VAT_RATE_UNAVAILABLE`,
   `PROVIDER_MOVEMENT_KIND_UNSUPPORTED`, `PROVIDER_AMOUNTS_INCONSISTENT`, `PROVIDER_ACCOUNT_REQUIRED`,
   `PROVIDER_PROPOSAL_REQUIRED`, et ceux de 38-6b (`PROVIDER_CONVERSION_PAIR_INVALID`, `PROVIDER_PAIR_ACCOUNT_LOCKED`,
   `PROVIDER_PAIR_CHANGED`, `PROVIDER_TRANSFER_*`, `PROVIDER_REFUND_ORIGIN_INVALID`) ; 38-6b : action d'audit
   `reconciliation.review_done`, `LIB_ROUTES` +1 ; 38-6c : `docs/api-external.md` dans son périmètre.
8. Récapitulatif FR/NFR à recompter **aux fiches** (grep), total réécrit.
9. Découpage D-P1-1 : 38-3a et 38-3b au tableau, fiches sœurs renvoyant à la bonne moitié ; `sprint-status.yaml` :
   `38-3-telechargement-cadre: split`, clés `38-3a-…` et `38-3b-…` `backlog`.

## Fiches sœurs

- **38-1** : retirer `accountEmail` partout (AC-4, E24, instantané d'audit, test) ; `PATCH` : `eventKindOverrides` et
  `overlapDays` modifiables par le client, `payerId` posé par le serveur seul ; 38-3 tranche `RESTRICT` ou suppression
  des passages (`provider_fetch_runs`) — à aligner une fois tranché en 38-3b.
- **38-2** : AC-1 déjà aligné sur O1 (P1) ; compléter avec le texte de 38-4 P1 (MODIFY `match_type` exact, CHECK
  `^[a-z][a-z_]{1,39}$`, index reconstruit à l'ALTER, P6 grep relevé, grep de `"0.17.6"` et tri, test d'unicité après
  ALTER, CHANGELOG, manuel 38-9a) ; AC-5 : `PROVIDER_ACCOUNT_IN_USE` gagne `balance_seen` (instantanés compris) ;
  `FEE_ACCOUNT_INVALID` porte aussi `missing` (`details.accountId` nul), posé par 38-6a.
- **38-3 → 38-3a/38-3b** : supprimer son fichier breaking (fait en P1, vérifier) ; `Connector` gagne
  `history_start(now)` et `overlap()` (le passage lit `overlap()` au lieu de −3 jours) ; `MovementPage` gagne
  `skipped { denied, pending, non_balance }` et `sums`, une page unique admise (`total_pages = 1`) ; `ConnectorError`
  gagne `WindowTooLarge`, `Rejected`, `InvalidWindow`, le passage écrit `error_code = err.code()` et `lastError` porte
  `app_code()` ; `HttpTransport` à bornes par requête (PayPal 60 s, 8 MiB) ; la fabrique passe le secret ouvert,
  `secret_updated_at`, le cache de jeton et les `payerId` des autres connexions PayPal ; `provider_fetch_runs` gagne
  `truncated_from DATETIME(3) NULL` et `ignored_count`, rendus par `GET …/runs` ; extension de `TransactionResponse`
  portée par 38-6c, non par 38-3 ; délai : bornes par requête (fin du 30 s/60 s contradictoire) ; trancher
  `RESTRICT`/suppression des passages.
- **38-6a** : C-38-6a-3 : le DTO de transaction des propositions porte `invoiceRef` et `eventCode`, le candidat
  `rateSource`.
- **38-6b** : AC-1 : le candidat `conversion` porte `pairedTransaction { bookingDate, amount, currency,
  bankAccountName }`, la vue de l'AC-8 de même pour `pairedBankTransactionId`.
- **38-7** : AC-7 aligné sur 38-3b (pas de statut HTTP sur `POST /fetch` : déjà fait en P1, vérifier).
- **38-8** : AC-4/AC-5 alignés sur O11 (déjà fait en P1, vérifier).
- **38-9a** : afficher `skipped`, `truncated_from`, `ignored_count` (si 38-3b les porte) et `overlapDays` aux réglages
  avancés.
- **38-9b** : libellés des codes du lot (`ENTRY_NOT_LINKABLE`…, `failed-proposal-label.ts`, quatre locales) portés par
  38-9b ; Q-38-9b-1 close par 38-6c (500 au plus, champs de `link-suggestions` à l'AC-4 de 38-6c).

## Epic 37 (branche `epic/37-multi-devises`, worktree `../kesh-wt37`)

- **37-1** AC-4 et C-37-9 : `ACCOUNT_IN_FOREIGN_CURRENCY` ne s'applique plus à
  `reconciliation_rules.counterparty_account_id` (levé par 38-4, O3) ; `ACCOUNT_CURRENCY_IN_USE` reste ; décomptes
  « dix » des colonnes désignées → « onze » quand 38-5 ajoute `default_vat_acquisition_account_id` — écrire ces deux
  points comme **notes de dépendance future** (« l'Epic 38 lève… / porte à onze ») sans changer le comportement de 37.
- **37-5** : le contrôle de clôture futur « comptes en devise à réévaluer » réservé en **K9** devient **K10** (38-10
  livre K9 `ATTACHED_STATEMENT_GAP`).
