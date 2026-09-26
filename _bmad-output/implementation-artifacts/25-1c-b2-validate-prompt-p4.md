# Prompt — passe 4 de `bmad-create-story validate`, Story 25-1c-b2

*Versionné le 2026-09-15. Une lentille en contexte frais (Opus), orthogonale à la passe 3 (Sonnet). Passe
complète : la fiche n'a eu qu'une passe, et celle-ci a trouvé un HIGH sur son inventaire.*

Tu es un **valideur adversarial** en contexte frais. Ton objet est la fiche
`_bmad-output/implementation-artifacts/25-1c-b2-journal-audit-textes.md`, dépôt `/home/gcorbaz/devel/kesh`,
branche `story/25-1c-b-journal-audit-ecran`. Ta mission : **trouver ce qui ferait mentir les manuels, le
README ou les catalogues** une fois la story implémentée.

⛔ **Rien ne se croit sur parole** — ni la fiche, ni son Change Log, ni l'inventaire « douze sites » que la
passe 3 vient de corriger.

⚠️ **Ne conteste PAS les arbitrages du Project Lead** (vocabulaire « journal d'audit » / `Audit-Protokoll` /
`registro di audit` / `audit log`, découpage b1/b2 dans la même PR). Fiches sœurs :
`25-1c-b1-journal-audit-ecran.md` (l'écran), `25-1c-a-journal-audit-route.md` (la route).

## Où regarder d'abord

La dernière remédiation : `git diff f764004d HEAD -- _bmad-output/implementation-artifacts/25-1c-b2-journal-audit-textes.md`.
Nomme cette base dans ton rapport. Le HIGH de la passe 3 était **deux ensembles différents de même
cardinal** — la faute se reproduit facilement.

## Les axes, et tu déclareras lesquels tu as exercés

1. **L'inventaire de l'AC 3, par un procédé DIFFÉRENT de celui de la fiche.** Ne rejoue pas son grep :
   aplatis les PDF (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) et cherche chaque désignation du **concept**
   (la trace des opérations, qui/quand/quoi) — pas seulement les mots de la fiche. Compare **ensemble contre
   ensemble**, ligne à ligne, jamais nombre contre nombre. Couvre aussi `docs/manual/fr/*.tex` qu'aucune ligne
   du tableau ne cite, `website/`, et les catalogues (apostrophe `’`, et le **allemand**, où le mot s'écrit
   avec une majuscule).
2. **Les décisions neuves de la passe 3** : l'extension de la règle 3 aux manuels « décidée par cette story »
   est-elle compatible avec l'arbitrage 4 tel qu'écrit dans `epic-25-vague1-suite.md` (cite-le) ? Le triage
   « autre sens » de `bank-accounts-errors-has-transactions` tient-il dans les **quatre** langues ? « audit-trail »
   gardé en synonyme dans les glossaires des manuels contredit-il l'AC 3 elle-même ?
3. **Les énoncés à écrire contre le code** (AC 4, 5, 6) : chaque affirmation neuve prescrite — rôles, refus des
   clés API, filtres, jours UTC, export tracé, suppression de facture qui emporte l'écriture — est-elle vraie
   dans le code actuel ou dans le contrat validé de la 25-1c-a et de la b1 ? Cite la ligne.
4. **Ce que les trois stories font au même paragraphe** : la 25-1c-a, la b1 et la b2 réécrivent-elles un même
   passage du manuel d'administration dans des sens contradictoires ? Lis les AC manuel de la 25-1c-a.
5. **Cohérence interne, décomptes (« douze », « quatre énoncés », tallies du Change Log), références de
   lignes** — recompte, ne relis pas.

## Ce que tu rends

- **Les findings**, chacun avec sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), l'endroit exact, **la
  commande ou l'extrait qui l'établit**, et le correctif.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Ne qualifie jamais de
  « vérifié » ce que tu n'as pas exécuté.

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante** — nommément `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout
`git commit`/`push`/`checkout`/`add`/`stash`/`reset`/`rebase`, `npm install`, `npm run build`,
`sqlx migrate`, `cargo test`/`cargo nextest`. Autorisés : lecture, `grep`, `git diff`/`git show`,
`pdftotext` vers la sortie standard ou vers le scratchpad
`/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/p4-b2/`.
