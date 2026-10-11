# Prompt — validation P3 de la spec, Story 26-2 — passe CIBLÉE (une lentille, Haiku)

*Versionné le 2026-10-10. Passe ciblée de fin de boucle (`CLAUDE.md` § Review Iteration Rule → « La passe ciblée »,
D6 de la rétro 25 : Haiku réservé aux passes ciblées). **Une seule lentille**, contexte frais, lecture seule,
braquée sur **le seul commit de la remédiation P2**.*

Dépôt `/home/gcorbaz/devel/kesh-26-2`, branche `story/26-2`. **Commit à relire : `2746da61`** — lis-le **aplati**,
jamais avec un autre commit : `git show 2746da61` (un seul commit ; ne combine pas avec `f7f3e212` ni `b053cc66`).
Fiche : `_bmad-output/implementation-artifacts/26-2-description-compte-bancaire.md` (26-2a socle AC1-8 / 26-2b écrans
AC9-12). Registre : `_bmad-output/implementation-artifacts/epic-26-choix-autonomes.md` (C-26-2-26 à 30 neufs ;
amendements de C-26-2-3, 5, 6, 8, 11, 15, 20, 22). Rapports P2 : `/home/gcorbaz/devel/kesh-gate-logs/26-2-validate-p2-R.md`
et `…-p2-F.md` ; bilan P2 en fin de fiche (Change Log).

**Pourquoi une passe ciblée** : la P2 a convergé (P1 8 MEDIUM → P2 3 MEDIUM distincts, dont un seul né de la
remédiation P1). Ce qui reste à relire, c'est **ce qu'on vient d'écrire** — la remédiation P2 touche des AC et une
**règle d'affichage** (le menu d'import garde « UBS (CH…) » sans description).

## Lentille — chasseur de régressions de la remédiation P2

Pour chaque hunk de `2746da61` :

1. **La règle d'affichage (C-26-2-26)** : `bankAccountOptionLabel(a, ibanStyle)` — vérifie que **sans description,
   chaque site D1-D9 rend exactement son affichage d'aujourd'hui**. Confronte le tableau d'AC9 (colonne
   « Aujourd'hui ») au code actuel : `grep -nF "bankName" <fichier>` pour chaque site D1-D9, sortie copiée. Le
   format `'parens'` n'est-il employé que pour D7 ? L'E2E (b) (AC13) attend-il bien « UBS E2E (CH…) » sur
   `/bank-import` et « UBS E2E — CH… » sur `/reconciliation` ? Les mutations (M17) et le manuel (AC12) suivent-ils ?
2. **Onboarding (C-26-2-22 amendé)** : plus aucune phrase de la fiche (AC2, AC12) ni du registre ne présente comme
   **vécu** par l'utilisateur un changement de banque à l'onboarding ; le test de l'upsert et M5 restent.
   `grep -nF "onboarding" <fiche>` et relis chaque occurrence.
3. **Correction du manuel (C-26-2-30)** : les lignes 911, 921, 933, 1033 de `docs/manual/fr/user-manual.tex` citées
   existent et disent ce que la fiche leur prête (`sed -n` de chacune) ; les affirmations sur la QR-facture
   (compte principal, QRR si QR-IBAN, **sans référence** sinon) sont vraies du code :
   `crates/kesh-api/src/routes/invoice_pdf_service.rs` l. 222 et 321-331.
4. **`BankAccountPutFields` / `find_by_id_in_tx(tx, company_id, id)` (C-26-2-27, C-26-2-20)** : nombre de
   paramètres réel de `update_for_company` (`sed -n 458,466p crates/kesh-db/src/repositories/bank_accounts.rs`),
   précédent `crates/kesh-db/src/repositories/projects.rs:59` ; la fiche (AC2, T2.1) et le registre disent-ils la
   même chose ?
5. **Ensemble Unicode et ordre trim/contrôle (C-26-2-28)** : cohérence entre AC3, M14, M18, T3.2 et le registre ;
   l'exemple `"Épargne\n"` (accepté après `trim()`) est-il conforme à `str::trim` (`White_Space`) ?
6. **Types TS et i18n** : `UpdateBankAccountPayload.description` obligatoire partout dans la fiche ? Libellé
   « Description (optionnelle) » et aide aux quatre locales cohérents dans AC7, AC8 et le registre (C-26-2-29) ?
7. **Recomptes** : 14 AC, **20 mutations** (M1-M18, M8-bis, M16-bis), 10 sites, 30 choix — recompte depuis le texte
   (`grep -oE "^\| M[0-9]+(-bis)? \|" <fiche> | sort -u | wc -l`, etc.) ; le bilan P2 (24 LOW, 4 MEDIUM dont 3
   distincts, paires) concorde-t-il avec les deux rapports ?
8. **Propagation** : greppe les **valeurs** corrigées sur la fiche, le registre, `sprint-status.yaml` et
   `_bmad-output/planning-artifacts/epic-26-recette-v0-13.md` : `UBS2`, `survit`, `UBS — CH… » au lieu`,
   `seul changement visible`, `constante de colonnes`, `find_by_id_in_tx(tx, id)`, `649-650`, `Trois détecteurs`,
   `ISO 11649`, `réception`, `closes #622` — chaque occurrence restante doit être historique (bilans) ou légitime.

⚠️ **Discipline Haiku** (`CLAUDE.md` § Haiku-specific guardrails) : toute affirmation de présence ou d'absence =
sortie de `grep -nF` **copiée** ; pour un motif multi-ligne, `grep -nFA <N>`. Ne conclus pas à l'absence d'un
texte sur la foi d'un numéro de ligne du diff. Un finding qui dit « tel prescrit est absent » doit montrer le grep
qui le prouve **sur le fichier final**, pas sur le diff.

## Ce que tu rends

Findings numérotés avec sévérité (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie
copiée), correction proposée, et pour chacun : **né de `2746da61`** ou antérieur. ⛔ **La liste des axes exercés ET
non exercés** (les huit ci-dessus) — un « 0 finding » sans elle ne compte pas. Rapport dans
`/home/gcorbaz/devel/kesh-gate-logs/26-2-validate-p3-ciblee.md` ; dernier message : le chemin, le bilan, une ligne
par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh` et les scripts Synology), `make`,
`latexmk`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`/`reset`/`rebase`, `cargo`, `npm`, `npx`, `docker`,
`bash` sur un script du dépôt, `sha384sum` vers un fichier, `gh` en écriture, SQL. Autorisés : lecture, `grep`,
`sed -n`, `git log`/`show`/`diff`, `gh api` en lecture, `pdftotext` vers la sortie standard.
