# Story 35-5 : réévaluation des comptes en devise à la fin de l'exercice

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/35-multi-devises`. Epic 35, D5, C-35-7. Suit 35-4.
**Aucune migration attendue** (`rate_source = 'revaluation'` posé par 35-3a).*

## Story

En tant que comptable en multi-devises, je veux qu'à la fin de l'exercice Kesh me propose la réévaluation de mes
comptes de bilan en devise au cours mensuel du mois de clôture, et la passe en une écriture après mon contrôle, afin
que mon bilan présente ces comptes au cours de clôture et que l'écart aille aux différences de change.

## Critères d'acceptation

1. **AC-1 — Aperçu.** `POST /api/v1/fiscal-years/{id}/fx-revaluation/preview { rates?: {USD: "0.8846", …},
   accountIds?: [...] }` → par compte de **bilan** (`Asset`, `Liability`) en devise, imputable : solde en devise et solde
   CHF **cumulés depuis l'origine** jusqu'à la date de fin d'exercice, cours retenu (défaut : cours AFC du **mois de la
   date de fin d'exercice**, `lookup` 35-2 ; surchargeable par devise), CHF réévalué = round(solde devise × cours),
   écart, compte de contrepartie (gain si l'écart augmente l'actif ou diminue le passif, sinon perte) ; comptes
   présélectionnés : tous ceux de bilan en devise ; un cours manquant sans surcharge → la devise est signalée
   `RATE_UNAVAILABLE`, ses comptes exclus jusqu'à saisie ; comptes `Revenue`/`Expense` jamais proposés.
2. **AC-2 — Passation.** `POST …/fx-revaluation { rates, accountIds, version? }` recalcule (jamais de confiance dans
   l'aperçu envoyé), puis passe **une** écriture journal `OD`, datée du dernier jour de l'exercice, libellé « Réévaluation
   des comptes en devise au JJ.MM.AAAA (écarts non réalisés) », une ligne par compte d'écart non nul
   (`rate_source = 'revaluation'`, `amount_currency = 0`, `exchange_rate` = cours retenu, CHF = |écart|) et les lignes
   de contrepartie CHF sur les comptes de gains et de pertes (une par compte de contrepartie et par sens, agrégée) ;
   aucun écart non nul → 200 sans écriture (`{ entryId: null }`).
3. **AC-3 — Rejouable.** Un second passage, sans mouvement entre-temps, ne passe rien ; après une écriture tardive en
   devise, il ne passe que le résidu. Pas d'extourne au premier jour de l'exercice suivant.
4. **AC-4 — Gardes.** Exercice clos → `FISCAL_YEAR_CLOSED` ; livres verrouillés à cette date
   (`books_locked_through`) → `PERIOD_LOCKED` ; exercice postérieur clos → l'erreur existante ; mono-devise →
   `MULTI_CURRENCY_DISABLED` ; comptes de différences de change revalidés à l'usage (`FX_ACCOUNT_INVALID`, C-35-2) ;
   `accountIds` hors de la liste admissible → 400.
5. **AC-5 — Audit** : l'écriture est auditée comme toute écriture ; l'aperçu n'écrit rien.
6. **AC-6 — Écran** : depuis *Exercices* (exercice ouvert), action « Réévaluer les devises » : tableau de l'aperçu,
   cours modifiables par devise (source affichée : AFC ou manuel), cases par compte, total des gains et des pertes,
   « Passer l'écriture », lien vers l'écriture passée ; captures d'écran.
7. **AC-7 — Tests** : deux exercices (achat EUR en N-1, réévaluation N-1, mouvements N, réévaluation N) : solde CHF au
   31.12.N = round(solde EUR × cours) exactement ; exercice clos en juin → cours de juin ; rejeu sans écriture ; compte
   décoché ou de résultat : aucune ligne ; cours surchargé figé sur la ligne ; refus des gardes ; E2E de l'écran.
8. **AC-8 — Manuel** : réévaluation, cours du mois de clôture (choix assumé, D5), gains et pertes non réalisés
   comptabilisés (CO 960b, choix assumé ; la neutralisation par provision n'est pas offerte, C-35-8), postes non
   monétaires à décocher ; i18n ; PDF.

## Tâches

- [ ] `kesh-core` : calcul pur de la réévaluation (entrées : soldes, cours ; sortie : lignes) et tests.
- [ ] `kesh-db`/`kesh-report` : soldes cumulés à deux devises à une date (fonction de 35-4).
- [ ] `kesh-api` : routes (aperçu, passation), gardes, i18n des erreurs.
- [ ] Frontend : écran depuis les exercices, Vitest, E2E.
- [ ] Manuel, CHANGELOG.

## Notes de développement

- Kesh ne passe aucune écriture de clôture (`crates/kesh-report/src/opening.rs:17`) : les soldes de bilan sont
  cumulés depuis l'origine ; d'où le calcul « jusqu'à la date », toutes années confondues (risque relevé à la table
  ronde).
- Exercices : `repositories/fiscal_years.rs` (`close` :1116), `routes/fiscal_years.rs` (:352) ; garde « exercice
  postérieur clos » `journal_entries.rs:368`.

## Change Log

- 2026-11-14 — fiche rédigée.
