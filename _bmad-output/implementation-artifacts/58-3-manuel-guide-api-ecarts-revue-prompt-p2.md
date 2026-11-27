# Story 58-3 — prompt de la revue de code P2 (Sonnet, contexte frais)

*Rédigé le 2026-11-27 à l'issue de la revue P1 (Opus 5.5 : 0 CRITICAL / 0 HIGH / 3 MEDIUM / 11 LOW ; rapport
`/home/gcorbaz/devel/kesh-gate-logs/e58/58-3-revue-p1.md`). Passe complète, non ciblée : la remédiation de la P1 touchera les deux manuels et,
selon le choix fait pour L5 et L7, `crates/kesh-db/src/post_restore.rs` ou un test de `kesh-api`.*

---

Tu conduis la **revue de code P2 de la story 58-3 de Kesh** (logiciel de comptabilité suisse en production ; Epic 58, nettoyage : écarts entre
le manuel, le guide de l'API et le code), avec le workflow BMAD `bmad-code-review` (invoque la compétence et suis-la). Contexte frais.

- Worktree `/home/gcorbaz/devel/kesh-58-3`, branche `story/58-3-manuel-guide-api-ecarts`. **Tête** : le dernier commit de la remédiation de la P1
  (le lire par `git log -1`) ; **base de comparaison** : `56d47b2cc` (le `main` repris). Si la branche a repris `main` depuis (`0e99cd2db`, v0.20.0),
  la base devient ce `main` : le dire.
- Fiche : `_bmad-output/implementation-artifacts/58-3-manuel-guide-api-ecarts.md` (critères AC1 à AC12, Dev Agent Record, Change Log).
- Rapport de la P1 : `/home/gcorbaz/devel/kesh-gate-logs/e58/58-3-revue-p1.md`. **Ne le lis qu'après avoir fait ta propre lecture du diff**, pour
  vérifier ensuite que chacun de ses findings MEDIUM et LOW est traité (corrigé, reclassé avec raison, ou renvoyé en issue).
- Lis le `CLAUDE.md` du worktree : *Review Iteration Rule*, *Le prompt d'une passe doit NOMMER le manuel*, *Une passe déclare ses axes exercés*,
  *Propagation post-patch*, *Recompter*, *Migration breaking policy* P5 et P7.

## Axes obligatoires (chacun déclaré exercé ou non exercé)

1. **Chaque critère AC1 à AC12 contre le code** : la phrase corrigée dit-elle ce que fait le code ? Lis le code cité, ne te fie pas à la fiche.
   En particulier :
   - AC1 : la ligne du rôle Consultation (`admin-manual.tex`, tableau RBAC) et le point « Portée » des clés. Une énumération qui compte (« trois
     gestes », « quatre choses ») doit être exhaustive pour son sujet : cherche toute écriture au journal d'audit ou en base qu'un rôle Consultation
     (session) ou une clé `read` peut provoquer (`crates/kesh-api/src/lib.rs`, groupes de routes « tout rôle authentifié » ; `routes/auth.rs` :
     `auth.password_changed`, `auth.password_reset_*`, `auth.session_theft_detected` ; `audit_labels.rs`).
   - AC5b : la phrase sur ce qu'éprouve le test de contrat du serveur MCP — `crates/kesh-api/tests/mcp_contrat.rs` contre
     `tests/mcp_outils_e2e.rs` et les tests de `src/mcp/sortie.rs` : quel test prouve quoi ?
   - AC7 : le compte de produit par défaut — `crates/kesh-db/src/repositories/invoices.rs` (`priced_line_without_account`,
     `validate_line_revenue_accounts_in_tx`, la ligne qui choisit explicitement le compte par défaut, et l'exigence du réglage).
   - AC4 : nom du fichier de log et commande `tail` (rotation `daily`, `hourly`, `never`, chemin sans suffixe ; création du fichier à la première
     écriture : `crates/kesh-api/src/logging.rs`, `~/.cargo/registry/src/*/tracing-appender-0.2.5/src/rolling.rs`).
2. **Le manuel, PDF compris** : régénère `admin` et `user` dans une **copie** de `docs/manual` sous `/tmp/claude-1000/` (jamais dans le worktree ;
   `cp -r docs/manual <copie> && make -C <copie> admin user`), **et** une copie construite depuis la base (`git archive <base> docs/manual | tar -x -C <copie-base>`).
   Aplatis (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`, puis `sed "s/’/'/g"`) et cherche chaque chaîne des AC, avant et après. **Compare les listes
   d'Overfull** des deux journaux (`grep -oE 'Overfull \\hbox \([0-9.]+pt too wide\) in paragraph at lines [0-9]+--[0-9]+'`, puis `diff`) : la story ne doit
   en introduire aucun dans un paragraphe qu'elle a écrit, et tout décompte d'Overfull écrit dans la fiche doit se retrouver.
3. **`docs/api-external.md`** : les deux lignes d'erreur (`ACCOUNT_NOT_POSTABLE`, `DESIGNATED_ACCOUNT_WRONG_TYPE`) contre `crates/kesh-db/src/errors.rs`
   (tri par numéro en chaîne, identifiant, réglage — ordre de l'enum) ; intégrité du tableau Markdown.
4. **`.env.example`, `docker-compose*.yml`, `crates/kesh-api/src/config.rs`** : commentaires seulement ? Aucune valeur ni ligne d'affectation touchée ;
   le commentaire de `.env.example` que lit `controle_raisons_valeurs` (`crates/kesh-api/tests/configuration_transmise.rs`) reste conforme.
5. **`crates/kesh-db/src/post_restore.rs`** : les deux justifications d'exemption réécrites (20260909000001, 20260915000001) ; les tests qui lisent ces
   textes (`every_exemption_declares_a_coherent_basis`, `exemptions_claiming_out_of_window_really_are_out_of_window` et ses voisins :
   `grep -n 'starts_with\|contains(' crates/kesh-db/src/post_restore.rs`) ; une justification « Hors fenêtre » doit commencer par cette chaîne, et le
   doc-comment au-dessus de `EXEMPT_MIGRATIONS` dit qu'une justification qui *invoque* la fenêtre doit commencer par elle : le texte et le doc-comment
   se contredisent-ils encore ? Seules des lignes de littéraux (ou de commentaires) changent-elles ?
6. **`docs/migrations-idempotence-audit.md`** (P5) : `ls crates/kesh-db/migrations/*.sql | wc -l` = ``grep -c '^| `20' docs/migrations-idempotence-audit.md`` ;
   en-tête `## Table d'audit (N migrations)` et ligne `Total` ; colonne verdict des lignes touchées inchangée.
7. **Reprise de `main`** : fusion `2c8ba7363` ; et, si elle a eu lieu, la reprise de `0e99cd2db` (v0.20.0). Sinon, fais une fusion d'essai **sans écrire** :
   `git merge-tree --write-tree --name-only HEAD 0e99cd2db`. Un conflit est attendu sur le point « Portée » de `admin-manual.tex` (la v0.20.0 en change
   la fin, la story le début) : la résolution doit garder les deux. Ce que la v0.20.0 a changé (`git diff 56d47b2cc 0e99cd2db -- docs`) rend-il une
   correction de la story fausse ?
8. **Méthode** : relis en entier (word-diff) les diffs des fichiers touchés par la remédiation, et ceux qu'avaient produits un script Python et un `sed`
   (`admin-manual.tex`, `api-external.md`) : aucun changement hors des sites attendus, ni d'encodage, ni de fin de ligne.
9. **Propagation** des valeurs et phrases changées sur tout le dépôt (code, doc-comments, tests, manuels, guide, README, `website/`, hors
   `_bmad-output/`) ; en particulier `crates/kesh-api/tests/audit_route_registry.rs` (« Aucune route `GET` n'écrit au journal ») et le symptôme de chaque
   finding corrigé par la P1. Grep de la valeur, pas de la formulation.
10. **Fiche** : le Dev Agent Record et le Change Log n'affirment que ce qui a tourné ; tout décompte (Overfull, lignes, fichiers) se recompte, avec son
    périmètre ; les findings de la P1 non corrigés sont reclassés avec raison ou nommés comme issues à ouvrir (une issue par problème).

Pour tout finding CRITICAL ou HIGH qui affirme l'absence d'un code ou la présence d'un anti-motif : `grep -nF` de vérification avant de le retenir.

## Tests

Tu peux lancer des tests ciblés sur la station :
`flock /tmp/station-heavy.lock flock /home/gcorbaz/devel/kesh-gate-logs/.gate.lock scripts/mem-guard.sh cargo nextest run -E 'binary(<x>)'`,
base `kesh_e583` (`CREATE DATABASE IF NOT EXISTS` seulement, migrée et semée). **Jamais `GRANT`/`REVOKE`, jamais de redémarrage ni de recréation du
conteneur.** Binaires pertinents : `textes_coherents`, `mcp_matrice_manuel`, `configuration_transmise`, `audit_route_registry`, et la lib `kesh-db`
(`-E 'package(kesh-db) and kind(lib)'`) si `post_restore.rs` a changé.

## Interdits

- N'applique **aucun** correctif. N'écris rien hors de `/tmp/claude-1000/` et de `/home/gcorbaz/devel/kesh-gate-logs/e58/` (sauf le prompt de la passe
  suivante, dans le worktree, commité seul).
- Ni `gh` en écriture, ni `scripts/prepare-release.sh`, ni aucun script mutant, ni `docker`, ni `git merge`/`rebase`/`push` hors du commit du prompt
  suivant ; rien sous `~/kesh/` ; **jamais `pkill`, `killall` ni `kill` par motif**.

## Rendu

Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e58/58-3-revue-p2.md` — findings classés (CRITICAL / HIGH / MEDIUM / LOW) avec preuve (ligne lue citée),
faux positifs écartés, **axes exercés et non exercés**. S'il reste un finding au-dessus de LOW, prompt de la passe P3 (Opus) à côté de celui-ci
(`…-revue-prompt-p3.md`), commité seul (`git commit -- <chemin>`, message en français, `refs` des issues de la fiche, aucun mot-clé de fermeture,
ligne `Co-Authored-By` de ton modèle) et poussé sur la seule branche `story/58-3-manuel-guide-api-ecarts`.
Réponse finale en français : décompte, une ligne par finding MEDIUM ou plus, axes exercés et non exercés, chemin du rapport.
