# Story 49-2 — prompt de la revue de code P2 (Sonnet, passe complète, contexte frais)

Tu conduis la **passe P2 de la revue de code** de la story 49-2 de Kesh (réordonnancement des lignes d'une facture client
en brouillon, boutons ↑ / ↓, #1756), avec le workflow BMAD **`bmad-code-review`** (invoque la compétence et suis-la).
Kesh est en production : Guy émet de vraies factures avec lui. **Passe complète**, non ciblée : la P1 (Opus 5.5) a rendu
**0/0/2/11** et les remédiations touchent le manuel, le guide de l'API, le README et peut-être `InvoiceForm.svelte`.

- Worktree `/home/gcorbaz/devel/kesh-49-2`, branche `story/49-2-reordonnancement-lignes` ; base `0e99cd2db` (v0.20.0) ;
  diff complet `git diff 0e99cd2db..HEAD`. Fiche : `_bmad-output/implementation-artifacts/49-2-reordonnancement-lignes.md`
  (Dev Agent Record, Change Log). Rapport de la P1 : `$GATE_LOGS/e492/49-2-revue-p1.md` — lis-le, mais **ne t'y fie pas** :
  revérifie chaque finding au code.
- Captures : `$GATE_LOGS/e492/captures/` (C1 à C6) — **regarde les images**.
- Lis le `CLAUDE.md` du worktree : *Review Iteration Rule*, *Recette visuelle*, *Le prompt d'une passe doit NOMMER le
  manuel*, *Propagation post-patch*, *Numérotation des versions*, *Documentation à synchroniser*.

## Ce que la P1 a trouvé, à vérifier comme résolu (ou non)

- **M1** — `docs/manual/fr/user-manual.tex:1572` : le paragraphe « Réordonner les lignes » coupait la description de la ligne
  de commentaire ; « Elle s'imprime sur toute la largeur du PDF… » suivait la phrase de l'avoir. Vérifier au **PDF**
  aplati que chaque « Elle » a le bon antécédent, et que la phrase « ne se réordonne pas : elle ne se réordonne qu'après »
  ne se contredit plus.
- **M2** — `docs/api-external.md` et `README.md` nommaient **v0.21.0** alors que #1756 est au milestone **`v0.20.1`** et
  que la règle a.b.c (CLAUDE.md § *Numérotation des versions*) ne fait monter `b` qu'avec une migration ou une rupture
  d'API. Vérifier le numéro écrit (`gh issue view 1756 --json milestone`, lecture seule) et greper `\bv?0\.21\.0\b` sur
  tout le dépôt (fiche comprise).
- **LOW** L1 à L11 : voir le rapport ; ceux qui ont été corrigés se relisent, ceux renvoyés en issue se vérifient tracés
  (lecture seule : `gh issue list --search …`).

## Commits de la reprise et de la remédiation (après `190f70eaf` et la P1 `f4faf30ec`)

- `a5205bf0f` — fusion de `origin/main` (Epic 55, `b41d045aa`) : `sitesTotal` 3958 + 5 + 7 = 3970 (garde verte,
  `$GATE_LOGS/e492/remediation/garde-sites-1.log`). Relire le **diff combiné** de cette fusion.
- `46b3c2ed9` — M1 (manuel), M2 (v0.20.1 au guide, au README, à la fiche), L5 (guide), L11 (README).
- `2f795d0a7` — code : L4 (DE), L7, L8 (`ignorerRepetition`), L10 (`reloading` au verrou), poubelle verrouillée et libellé
  `invoice-line-remove` (`sitesTotal` 3971), L9 et L1 (E2E : `pdftotext`, touche Espace, `expect.poll`, `animations:
  'disabled'`), sélecteurs de l'E2E par `data-move` (garde KF-043 venue de `main`).
- Les commits suivants (captures, fiche) : `git log 190f70eaf..HEAD`.
- Preuves : `$GATE_LOGS/e492/remediation/` (`rouge-L7-L8-L10-poubelle.log`, `retrait-mutations.log`,
  `manuel-grep-2.log`, `front-2.log`).

## Axes obligatoires — chacun déclaré exercé ou non exercé

1. Chaque AC (AC1 à AC14) contre le code et les tests.
2. **Une facture validée ne se réordonne jamais** — écran (fiche sans boutons, C5) et serveur (`409
   ILLEGAL_STATE_TRANSITION`, T5) ; le PDF figé fait foi.
3. Perte de données en déplaçant une ligne : prix ambigu ou illisible et sa rangée de message, taux, compte de produit,
   type `comment`.
4. Concurrence : `409 OPTIMISTIC_LOCK_CONFLICT`, « Recharger », « Annuler » pendant un rechargement (L10), envoi en cours.
5. Accessibilité : focus après déplacement (y compris Entrée maintenue, L8), `aria-live` (réannonce d'un même texte, L7),
   noms des boutons renumérotés.
6. i18n des quatre locales : sens des libellés DE / IT / EN (L4).
7. **Le manuel, PDF compris** : régénère `user` dans une **copie** de `docs/manual` sous `/tmp/claude-1000/` (`make user`
   sous `scripts/mem-guard.sh`), aplatis (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) et grepe les phrases neuves ;
   aucun PDF ne se commite.
8. Le guide de l'API (`docs/api-external.md`, § des lignes d'une facture client) contre le code (`is_no_op_change`,
   `update`, `insert_lines` dans `crates/kesh-db/src/repositories/invoices.rs`).
9. Aucun code de production hors du domaine `invoices` du frontend et des catalogues :
   `git diff --stat 0e99cd2db..HEAD -- 'crates/*/src/*'` et les hunks de `invoices.rs` tous dans `mod tests`.
10. Toute remédiation postérieure à la P1 : relue en entier (`git log 190f70eaf..HEAD -p`), propagation grepée.

## Tests autorisés

- vitest d'un fichier : `flock $GATE_LOGS/.gate-front.lock npx vitest run <fichier>` (depuis `frontend/`).
- Rust ciblé : `flock /tmp/station-heavy.lock flock $GATE_LOGS/.gate.lock scripts/mem-guard.sh cargo nextest run -E
  'binary(invoice_line_types_e2e)'`, base `kesh_e492`.

## Interdits

Rien hors du worktree, de `/tmp` et de `$GATE_LOGS/e492/` ; **aucun correctif** ; ni `gh` en écriture, ni
`scripts/prepare-release.sh`, ni aucun script mutant, ni `docker`, ni `sed -i` ; jamais `GRANT` / `REVOKE` ; jamais de
redémarrage du conteneur ; rien sous `~/kesh/` ; jamais `pkill`, `killall` ni `kill` par motif.

## Rendu

Rapport `$GATE_LOGS/e492/49-2-revue-p2.md` : findings classés (CRITICAL / HIGH / MEDIUM / LOW) avec preuve (fichier:ligne,
extrait, sortie de commande), **axes exercés et non exercés** (un « 0 finding » sans cette liste ne compte pas), et s'il
reste un finding au-dessus de LOW, le prompt de la passe P3 (Opus) versionné à côté de la fiche.
