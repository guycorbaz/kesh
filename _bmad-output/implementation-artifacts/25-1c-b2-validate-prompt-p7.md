# Prompt — passe 7 CIBLÉE de `bmad-create-story validate`, Story 25-1c-b2

*Versionné le 2026-09-15. Une seule lentille en contexte frais (Sonnet), orthogonale à la passe 6 (Opus).
Avant-dernière passe autorisée par le plafond de 8.*

Tu es un **chasseur de régressions** en contexte frais. Ton objet est la fiche
`_bmad-output/implementation-artifacts/25-1c-b2-journal-audit-textes.md`, dépôt `/home/gcorbaz/devel/kesh`,
branche `story/25-1c-b-journal-audit-ecran`.

**Ton périmètre est la remédiation de la passe 6**, et seulement elle :
`git diff 06fda3b0 -- _bmad-output/implementation-artifacts/25-1c-b2-journal-audit-textes.md`. Nomme cette
base dans ton rapport.

⚠️ **Le motif à chasser** : les deux MEDIUM de la passe 6 étaient des défauts de **propagation** — une
phrase corrigée, sa jumelle ailleurs laissée intacte. Pour **chaque** phrase modifiée par le diff, cherche
dans toute la fiche (corps, tâches, Dev Notes) **une autre phrase qui dit la même chose** et vérifie qu'elle
dit maintenant la même chose. Greppe les **valeurs et les notions**, pas les formulations.

## Les axes, et tu déclareras lesquels tu as exercés

1. **Propagation de chaque modification** :
   - la conclusion de l'AC 4 (« se réécrit, ou se remplace par une note », premier énoncé conservé) contre
     son introduction, le tableau et la tâche T3 ;
   - le décompte de cellules de l'AC 2 contre les Dev Notes ;
   - le reste attendu de l'AC 8 contre la liste des tris de l'AC 3 et la tâche T5 ;
   - l'état de #380 contre toute autre mention d'issue ;
   - la note de l'AC 3 sur la brochure.
2. **Le rejeu de l'AC 8 est-il exécutable tel qu'écrit ?** Exécute-le en Python sur les PDF aplatis actuels et
   le README : le recollage `re.sub(r"(\w)- (\w)", r"\1\2", t)` recolle-t-il à tort un tiret légitime
   (« Comptable- Admin », un intervalle, un tiret cadratin mal aplati) au point de créer ou masquer une
   occurrence du motif ? Le « reste attendu » écrit en entier correspond-il, **aujourd'hui**, à ce que rend
   le motif sur les sites qui ne doivent pas changer ?
3. **L'état des issues citées** : `gh issue view 380 --json state,closedAt` et `381` — la fiche dit-elle vrai ?
   Le commentaire du code (`crates/kesh-db/src/repositories/journal_entries.rs` autour de 965-990) cite-t-il
   bien ces deux numéros ?
4. **Le Change Log de la passe 6** : décomptes recomptés contre sa liste.

## Ce que tu rends

- **Les findings**, chacun avec sévérité (`CRITICAL` / `HIGH` / `MEDIUM` / `LOW`), l'endroit exact, **la
  commande ou l'extrait qui l'établit**, et le correctif.
- ⛔ **La liste des axes réellement exercés ET de ceux qui ne l'ont pas été.** Ne qualifie jamais de
  « vérifié » ce que tu n'as pas exécuté.

## Interdits

⛔ **N'écris aucun fichier du dépôt et n'exécute aucune commande qui écrit dans le dépôt ou dans une base
persistante** — nommément `scripts/prepare-release.sh`, `scripts/regen-test-schema.sh`,
`scripts/install-hooks.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`, `make`, `latexmk`, tout
`git commit`/`push`/`checkout`/`add`/`stash`/`reset`/`rebase`, `gh issue edit`/`comment`/`close`,
`npm install`, `npm run build`, `sqlx migrate`, `cargo test`/`cargo nextest`. Autorisés : lecture, `grep`,
`git diff`/`git show`, `gh issue view`, Python en lecture, `pdftotext` vers la sortie standard ou vers le
scratchpad `/tmp/claude-1000/-home-gcorbaz-devel-kesh/cb9f9ce3-4808-472f-93d0-698c43110e0e/scratchpad/p7-b2/`.
⚠️ `grep` est ici `ugrep`, qui refuse les motifs trop complexes : utilise Python pour les extractions à
contexte.
