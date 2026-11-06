# Prompt — revue de code P2 ciblée de la fiche `v0174-correctifs-1` (#819), une lentille, Sonnet

Passe **ciblée** (règle « la passe ciblée » du `CLAUDE.md`) : une seule lentille, en contexte frais, braquée sur **la seule
remédiation de la revue P1** de la fiche 1 de Kesh (0.17.4, import CAMT.053, sans migration). Modèle : **Sonnet**. Langue :
français. Rapport (le **seul** fichier que tu écris) : `/home/gcorbaz/devel/kesh-gate-logs/v0174c-review-p2-1.md`.

Worktree `/home/gcorbaz/devel/kesh-0174c`, branche `story/v0174-correctifs`. La remédiation, aplatie :
`git -C /home/gcorbaz/devel/kesh-0174c show f3b5d5b1 -- crates/kesh-import crates/kesh-api/tests/bank_imports_e2e.rs CHANGELOG.md docs/manual/fr/user-manual.tex`
et le PDF `docs/manual/fr/user-manual.pdf` (régénéré après). Les findings P1 visés :
`/home/gcorbaz/devel/kesh-gate-logs/v0174c-review-p1-1-{B,E,A}.md` — MEDIUM B-1 = E-4 = A-1 (v08 en lecture corrigée),
LOW B-3, A-3, A-5, A-6, A-7. Fiche : `_bmad-output/implementation-artifacts/v0174-correctifs-1.md` (Change Log et Dev Agent
Record, entrée « Remédiation de la revue P1 »). Pièces, lecture seule : `kesh-gate-logs/v0174c/mut/MA-819-8.log`,
`mut/mutations.log`, `nextest-final.log`, `gate-final.log`, `e2e-full-3.log`, `pdf-controle2.txt`.

## Interdits absolus

- **Lecture seule** : aucun fichier hormis ton rapport, aucun fichier temporaire, **aucune redirection** (`>`, `>>`,
  `tee`, `2>&1`…). **Aucun `rm`**, aucun `git` qui écrit (`checkout`, `restore`, `commit`, `stash`, `reset`, `add`…).
- **Rien qui compile ou exécute** : ni `cargo`, ni `npm`/`npx`/`node`, ni `python`, ni `make`, ni `docker`, ni `curl`,
  ni `claude`, ni `sed -i`, ni **`scripts/prepare-release.sh`**, ni aucun script de `kesh-gate-logs`. **Aucune commande `gh`.**
- Permis : `git diff/show/log/grep`, `grep`, `sed -n`, `awk` (sans redirection), `cat`, `ls`, `wc`,
  `timeout 60 pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '` puis `grep -cF` à motif fixe.
- Toute affirmation de présence ou d'absence : une sortie `grep -nF` (ou `sed -n`) copiée.

## Axes

1. **La remédiation résout-elle chaque finding ?** Le test v08 (`v08_corrected_reading_resolves_entities_and_cdata_819`)
   passe-t-il bien par `v08::parse` (namespace, `Pty`), et la mutation MA-819-8 (`v08.rs` en `Legacy0173`) est-elle tuée
   par lui (journal) ? Le test des références de caractère invalides (`&#0;`, `&#xD800;`) rend-il `MalformedXml` pour la
   bonne raison (la branche `resolve_char_ref`, et non une erreur du lecteur) ? Le témoin (d) asserte-t-il la clé neuve ?
2. **Défaut neuf introduit par la remédiation** : la phrase du manuel et du CHANGELOG sur les deux limites (#915) dit-elle
   vrai au code (`legacy_0173_aliases`, `detect_duplicate_lines_with_aliases`) — conditions d'échec de la relecture, faux
   doublon « même jour, même montant » ? Le README des fixtures ? Le PDF porte-t-il la phrase neuve ?
3. **Le Dev Agent Record** dit-il exactement ce qui a tourné (A-5) ?

## Rapport

Findings (sévérité, fichier:ligne, preuve copiée, remède) ; liste des axes **exercés** et **non exercés** ; décompte
recompté.
