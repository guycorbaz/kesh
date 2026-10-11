# Prompt — revue de code P2 ciblée, Story 26-2a — commit de remédiation `fd7624a5`

*Versionné le 2026-10-11. **Une lentille** (Haiku), contexte frais, lecture seule — passe ciblée de fin de boucle
(`CLAUDE.md` § « La passe ciblée »). La P1 (Sonnet ×3, B/E/A) a rendu 0 CRITICAL, 0 HIGH, 0 MEDIUM, 11 LOW ; la
remédiation de ces LOW est le seul objet de cette passe.*

Dépôt `/home/gcorbaz/devel/kesh-26-2`, branche `story/26-2`. **Commit à relire : `git show fd7624a5`** (un seul
commit — n'utilise pas d'autre diff). Rapports P1 : `/home/gcorbaz/devel/kesh-gate-logs/26-2a-review-p1-{B,E,A}.md`.
Bilan P1 : fiche `_bmad-output/implementation-artifacts/26-2-description-compte-bancaire.md`, section « Revue de
code P1 — bilan ».

**Le motif à chasser** : *la sévérité se déplace vers ce qu'on vient d'écrire.* Une remédiation introduit-elle un
défaut ?

## Axes

1. `crates/kesh-api/src/exports/csv_tables.rs` : le test déplacé est-il entier, compile-t-il à sa nouvelle place
   (lecture), la doc `#262` est-elle bien rattachée à `serialize_invoices_csv_header_exhaustif`, l'en-tête attendu
   égal-il **exactement** l'en-tête écrit par `serialize_bank_accounts_csv` (compare colonne par colonne à
   `csv.write_record([...])` de la fonction) ? La continuation de chaîne `\` produit-elle bien un `;` sans espace ?
2. `crates/kesh-db/src/entities/bank_account.rs` : le test réduit prouve-t-il encore le séparateur U+2014 ?
3. `crates/kesh-api/src/routes/bank_accounts.rs` : la doc-comment modifiée dit-elle vrai — chaque point de code cité
   passe-t-il réellement `is_forbidden_in_description` (n'est ni `is_control`, ni dans une plage du `matches!`) ? Le
   code de la fonction a-t-il changé (il ne doit pas) ?
4. Fiche : chaque ligne du bilan P1 correspond-elle au diff ? La commande ERE recopiée au Dev Agent Record rend-elle
   ce qu'elle annonce (exécute-la : `grep` est permis) ?
5. Ce commit touche-t-il une **ligne de code de production** (hors commentaire) ? Réponds oui/non avec la preuve.

## Ce que tu rends

Findings numérotés (`H1-…`), sévérité, `fichier:ligne`, preuve (commande et sortie copiée, `grep -nF` pour toute
présence ou absence). Distingue un **défaut** d'un livrable prescrit absent. ⛔ **Liste des axes exercés ET non
exercés.** Rapport dans `/home/gcorbaz/devel/kesh-gate-logs/26-2a-review-p2-H.md` ; dernier message : le chemin, le
bilan, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit, compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`), `make`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/
`rebase`/`push`, `gh`, `cargo`, `npm`, `npx`, `docker`, `mariadb`, `sqlx`. Lecture seule : `git show`, `git diff`,
`git log`, `grep`, `sed -n`, `cat`.
