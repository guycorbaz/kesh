# Prompt — validation P3 ciblée de la spec, Story 15-1b-0

*Versionné le 2026-10-09. Passe ciblée (CLAUDE.md § « La passe ciblée ») : une seule lentille (Sonnet), contexte frais, en
lecture seule, braquée sur ce que le dernier commit de remédiation (`18f25973`) a écrit dans CETTE fiche.*

Dépôt `/home/gcorbaz/devel/kesh-15-1-suite`, branche `story/15-1-suite-du-lettrage` (base `056997b0`). **Objet** :
`git show 18f25973 -- _bmad-output/implementation-artifacts/15-1b-0-propriete-des-lignes-par-lot.md`, puis la fiche à `HEAD`. Fiches voisines (contexte) :
`15-1a2-0-lettrage-fige-avec-la-periode.md`, `15-1a2-i-lettrage-des-pieces-clients.md`, `15-1a2-ii-fournisseurs-et-rattrapage.md`,
`15-1b-0-propriete-des-lignes-par-lot.md`, `15-1b-vue-lignes-ouvertes.md`. Registre : `epic-15-choix-autonomes.md`.
Rapports précédents : `/home/gcorbaz/devel/kesh-gate-logs/15-1b-0-validate-p*-*.md`.

⚠️ Un défaut est ce que la fiche prescrit de faux AU CODE de `056997b0` (ou de ce que les fiches voisines créent), une
contradiction avec une fiche voisine, ou un test qui ne prouverait pas ce qu'il dit. Ce qu'une AUTRE story prescrit
n'est pas un manque. Toute affirmation de présence ou d'absence : la sortie d'un `grep -nF`, copiée.

## Lentille unique — chasseur de régressions de la remédiation

1. **`DocumentRef.document_type` typé en `DocumentKind`** (C-15-1b-0-3) alors que `DocumentRef` est posé par la 15-1a2-i et
   consommé par la 15-1a2-ii, toutes deux mergées AVANT la 15-1b-0 : la tâche de re-typage et le re-grep au T0 sont-ils
   écrits de façon exécutable ? la sérialisation JSON (`documentType` d'audit) reste-t-elle identique octet pour octet
   (`as_str`) ? les tests d'audit des deux fiches la tiennent-ils vraiment ?
2. **Parité par lot** (F2-2) : achat et règlement d'une même facture fournisseur dans le lot, dérivée fournisseur en
   `UNION ALL`, mutation (e) — la comparaison aux rangs 3 à 7 de la requête gelée prouve-t-elle le chemin par lot ?
3. **Mesure par `Com_stmt_execute`** étalonnée par un `SELECT ?` lié : protocole décidable ?
4. **Deux gardes de la transaction unique de la route** (`modification_blocker` prend une `&mut Transaction` ; test
   lexical `get_journal_entry_reads_in_one_transaction`, mutation (f)) : mordent-elles ?
5. Recomptes (7 AC, 5 tâches, 11 tests neufs, six mutations).

## Ce que tu rends

Findings numérotés (CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve**, correction proposée. ⛔ **Liste des axes
exercés ET non exercés.** Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/15-1b-0-validate-p3-ciblee.md` ;
dernier message : chemin, bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*`, `make`, `git commit`/`add`/`checkout`/`switch`/`stash`/`fetch`, `sqlx`, `cargo`, `npm`, `npx`, `docker`,
`gh` en écriture, SQL. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`, `pdftotext` vers la sortie standard.
