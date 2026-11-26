# Prompt — validation Pv2p7 de la fiche 55-3a (ciblée, sur les décisions du mainteneur du 2026-11-26)

## Contexte

Kesh : comptabilité suisse (Rust/Axum, MariaDB, SvelteKit), en production chez son mainteneur. Epic 55, « Navigation de
l'interface » : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md`. Worktree
`<worktree>/kesh-e55`, branche `epic/55-navigation`, partie de `origin/main` à `1e23edb55`.
La PR de l'Epic 46 se lit par `git show pr/46-rapports-groupes:<fichier>` (`511e61d54`).

Passe **Pv2p7**, ciblée, **contexte frais**, modèle Opus. Langue du rapport : français. Le commit `cf0c2dcbd`
(`git show cf0c2dcbd`) verse les décisions du mainteneur du 2026-11-26 dans le fichier d'epic et les fiches. C'est la
**septième** passe de la version récrite de la 55-3a ; la 55-3b est au plafond de huit passes : cette passe la lit pour
la cohérence avec la 55-3a et le fichier d'epic, et ses points sur la 55-3b vont au T0 de la 55-3b (dis-les quand même,
avec leur sévérité).

## À lire d'abord

1. `CLAUDE.md` du worktree : § *Review Iteration Rule* et ses sous-sections, § *Test Locally First*, § *Travailler à
   plusieurs développeurs*.
2. Le fichier d'epic en entier ; la fiche `55-3a-justificatifs-serveur.md` en entier ; la fiche `55-3b-justificatifs-ecran.md`.
3. Le code cité, **ligne par ligne**, à la base indiquée.

**Les décisions du mainteneur à verser** (2026-11-26 ; c'est contre elles que la passe juge la fidélité) :
1. **Les PDF marqués sortent de l'Epic 55** et forment un petit epic à part, « PDF marqués » (numéro à confirmer) : le
   filigrane, l'en-tête et la QR barrée dans `kesh-qrbill` ; l'aperçu « APERÇU — non émis » ; le PDF du brouillon. Retirer
   de la 55-3a la route `GET /invoices/{id}/pdf/preview`, la marque, AC-9, AC-11, AC-12, T2quater et leur recette visuelle,
   **rangés sans être supprimés** dans une section « Reporté à l'epic « PDF marqués » ». L'Epic 55 garde la navigation,
   `hasDocuments`, la route qui liste les documents d'une écriture et l'icône : facture **figée** → l'icône ouvre le PDF
   figé ; facture **pas encore émise** → « PDF pas encore émis », **sans aperçu**, tant que l'autre epic n'est pas livré ;
   terme exact de Kesh ; texte aux quatre locales. Taille de la 55-3a recalculée (elle doit redescendre) ; 55-3b mise à
   jour (visionneuse, bandeau, cas « pas encore émis ») ; `pdfStatus` mis à jour si sa liste change ; énumération ouverte
   pour l'aperçu à venir et le PDF externe.
2. **Un PDF déjà sorti sans être figé** (`emailed_at`, ou une empreinte d'audit) : décidé pour l'epic « PDF marqués »
   (aperçu marqué, bandeau « Un PDF de cette facture est sorti le … sans être gardé ; ceci est un aperçu. »), rangé dans la
   section reportée. Pour l'Epic 55, une telle facture n'est pas figée → « PDF pas encore émis » ; dire s'il faut une
   nuance de texte.
3. **Ligne surlignée** : à l'epic à part des liens de compte (56 proposé), seulement depuis une écriture, une facture ou
   une transaction ; pas un livrable de la 55.
4. **Resserrement de la dérogation Q6** : à la clôture de chaque story écran qui touche `messages.ftl` ou le manuel,
   `nextest -p kesh-i18n` et `binary(textes_coherents)` ; suivi par la 55-2 (R-2) et les autres fiches concernées.
5. **Questions ouvertes** : ne garder que la numérotation (56 et « PDF marqués »).

**Passe ciblée** : une seule lentille, braquée sur le commit `cf0c2dcbd`. Juge la 55-3a, ce que le commit change au
fichier d'epic, et la 55-3b pour sa cohérence. Vérifie que chaque changement est juste au code, fidèle aux décisions (ni
plus, ni moins), sans défaut neuf, et propagé (même symptôme ailleurs : `grep` de la valeur — `preview`, `notFrozen`,
`not-frozen`, `APERÇU`, `D13`, `kesh-qrbill`, `XL`, `bandeau` — sur les fiches 55-* et le fichier d'epic, hors sections
reportées et historiques datées).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A1 — Prémisses relues au code** : notamment `crates/kesh-api/src/routes/issued_invoice_pdf.rs` (`get_or_freeze`,
  `serve_frozen`, `is_frozen`) — `GET /invoices/{id}/pdf` sur une facture figée n'écrit-il vraiment rien (fichier,
  colonne, audit) ? — `crates/kesh-db/src/repositories/invoices.rs` (dévalidation, `emailed_at`), `errors.rs:443`, les
  catalogues `crates/kesh-i18n/locales/*/messages.ftl` (termes « PDF émis », « figé » et leurs traductions). Recopie la
  ligne lue.
- **A2 — Fidélité aux décisions** : chaque décision versée exactement ; rien de reporté ne reste exigé dans l'Epic 55 ;
  rien de gardé n'a été retiré ; la section reportée contient bien tout le contenu retiré ; une question tranchée
  reste-t-elle écrite comme ouverte ?
- **A3 — Critères constatables** : AC-9 récrit, AC-10 complété, AC-3/AC-5/AC-8 de la 55-3b : comment on les constate.
- **A4 — Cas limites** : facture figée devenue non figée avant le clic (course), valeur inconnue de `pdfStatus`,
  champ absent, liste à deux éléments `notFrozen` puis `frozen`, facture annulée figée, PDF figé disparu (`410`).
- **A5 — Preuve du rouge** des tests modifiés (Vitest, E2E de la 55-3b ; AC-10 de la 55-3a).
- **A6 — Manuel** : la phrase du § 11.6.1 (AC-9 de la 55-3b) contre `docs/manual/fr/user-manual.tex:1635-1650` ; une
  phrase rendue fausse ?
- **A7 — Règles du projet** : DRY, i18n quatre locales, captures, splitting, note de version (`refs #`, aucun mot-clé de
  fermeture), § *Dérogation* resserré.
- **A8 — Taille** : la 55-3a ramenée à M est-elle juste (modules, AC, tâches restants) ?
- **A9 — Fichiers communs et en-têtes** : lignes du tableau du fichier d'epic, § *Backend*, en-tête, `sprint-status.yaml`
  (bloc de l'Epic 55) cohérents avec les fiches.
Sévérités : CRITICAL / HIGH / MEDIUM réservées à ce qui fausserait le développement, la vérification ou le produit ;
LOW pour la rédaction sans effet. Ne gonfle pas, ne minimise pas.

## Interdits

Lecture seule. N'écris **aucun** fichier du dépôt ni du worktree. Aucun `cargo`, `npm`, `make`, `docker`, `mariadb`,
`sqlx`, aucun `scripts/*` (en particulier ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni
`scripts/gate-ci.sh`, ni `scripts/mem-guard.sh`). Aucun `git commit`, `push`, `fetch`, `checkout`, `switch`, `stash`,
`merge`, `reset`, `worktree`. **Aucun `gh`.** Ne touche à aucun port ni service (ni 8888, ni 8889). `grep`, `git show`,
`git log`, `sed -n`, `pdftotext`, `python3 -I` en lecture sont admis.

## Rapport

Écris **un seul fichier** : `$GATE_LOGS/s55-decisions-2/55-3a-validation-pv2p7.md`, avec :
- le **verdict chiffré** CRITICAL / HIGH / MEDIUM / LOW, **séparément** pour la 55-3a, le fichier d'epic et la 55-3b ;
- chaque finding : identifiant (`Pv2p7-n`), fiche, sévérité, ce qui est faux, la preuve (ligne lue), le correctif proposé ;
- la **liste des axes exercés et non exercés** (une passe qui ne la donne pas ne compte pas).
Puis rends ce même contenu en réponse, en entier.
