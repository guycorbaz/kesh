# Prompt — revue de code P1 de la story 50-6 « Dates et heures de Zurich » (#1511), passe complète

Tu relis le **code** de la story **50-6** de Kesh (comptabilité suisse ; Rust/Axum, MariaDB, SvelteKit), Epic 50 « Corrections », release
**0.20.0**. Kesh tient les livres réels du Project Lead. **Aucune migration, aucun code serveur** : seul le frontend, ses tests, un
manuel `.tex` et la fiche changent. Première passe de revue : **complète** (jamais ciblée), contexte frais, modèle **Opus**. Langue du
rapport : français.

Worktree `/home/gcorbaz/devel/kesh-50-6`, branche `story/50-6-dates-heure-de-zurich`, partie de `origin/epic/50-corrections` à
`e5654fc00`. Fiche : `_bmad-output/implementation-artifacts/50-6-dates-heure-de-zurich.md` (statut `review`, Dev Agent Record rempli).
Le diff à relire : `git diff e5654fc00..HEAD` (tête = dernier commit de la branche ; lis-le **aplati**, un seul diff, non commit par commit).

## Ce que la story fait (à contrôler, non à croire)

Les instants enregistrés par Kesh (journal d'audit, sauvegardes — écran et bandeau —, envois et prompts du serveur MCP, rejet de
rapprochement) s'affichaient dans le fuseau du **poste** ; ils s'affichent désormais à l'heure de **Zurich**, par une fonction unique
`formatInstant(iso, 'date' | 'dateHeure', locale): string | null` de `frontend/src/lib/features/reports/reports.api.ts` (lecture
« naïf = UTC » factorisée dans `instantDuServeur`, formateurs en cache, `timeZone: 'Europe/Zurich'`). Quatre sites convertis ;
`mcp-prompts-format.ts` : un commentaire seulement. La garde `lib/shared/utils/local-date-guard.test.ts` gagne deux motifs et une
allowlist (`ALLOWLIST_INSTANTS`, champ `lignes?`). Le manuel d'utilisation (`user-manual.tex`) est corrigé. **Décisions du Project Lead
(2026-11-24), tranchées, hors de ta revue** : l'heure de Zurich partout où l'on affiche un instant ; changement voulu d'un comportement
documenté. Exception (dates **saisies** : expiration d'une clé MCP ou d'API) et limite (instants de l'écran *Serveur MCP*, issue I2 à
ouvrir) écrites dans la fiche : tu peux signaler qu'elles sont mal **appliquées**, pas les rediscuter.

## À lire d'abord

1. `CLAUDE.md` du worktree : § *Qualité du code*, *Recette visuelle*, *E2E (Playwright)*, *Review Iteration Rule* (dont *Inventorier les
   sites NON RÉSOLUS*, *Le prompt d'une passe doit NOMMER le manuel*, *Une passe déclare ses axes exercés*), *Priorités des défauts*.
2. La fiche en entier, puis `git diff e5654fc00..HEAD` aplati.
3. Journaux de la story, en lecture seule, dans `/home/gcorbaz/devel/kesh-gate-logs/e506/` : `ac1-json.txt` (JSON réel), `rouge-*.txt`
   (les rouges), `mut-M1.txt` à `mut-M4.txt` (mutations), `manuel-avant.txt` / `manuel-apres.txt`, `garde-releves-*.txt`,
   `front-gate.log`, `testfast.log`, `e2e-complet.log`, et les captures `captures/{avant-ny,apres-ny,apres-zurich}/*.png` (images : lis-les).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A1 — `formatInstant` et `instantDuServeur`** (`reports.api.ts`) : la regex `TIMESTAMP_RE` admet-elle toutes les formes que le serveur
  rend (`Z`, naïf à 3 décimales, 9 décimales, espace au lieu de `T`) ? Le cache `Map` (`locale|style`) peut-il servir un formateur faux ?
  `zurichParts` / `formatSwissDate` / `formatSwissDateTime` se comportent-ils **exactement** comme avant (leurs tests existants verts sans
  modification) ? `null` pour absent et illisible ; une date pure `AAAA-MM-JJ` passée à `formatInstant` rend-elle `null` (et est-ce
  voulu pour chaque appelant) ?
- **A2 — les quatre sites** : `audit-log/+page.svelte` (repli `?? e.createdAt`, `data-testid="audit-log-row-date"`, réactivité à
  `i18nLocale()`), `UnrejectTransactionAction.svelte` (`$derived`), `backup-format.ts` (`''`), `mcp-sends-format.ts` (`—`, valeur
  brute). Les contrats de repli sont-ils **inchangés** ? Un appelant dépend-il de l'ancien comportement (`McpSendsList`, `McpSendDetail`,
  `McpPromptDetail`, `AutoBackupSection`, `BackupScheduleBanner`) ? Le bandeau compare-t-il des instants (logique) ou seulement les affiche-t-il ?
- **A3 — inventaire par le symptôme, ensemble clos** : rejoue la commande de recompte de la fiche (D3) et **ta propre** recherche
  (motifs à écrire par toi, partant du symptôme : tout instant formaté ou lu sans fuseau) sur `frontend/src` : reste-t-il un site non
  résolu ni inscrit ? Inventorie les sites **non résolus** (fichiers `.svelte` de `lib/components/` et `lib/shared/` compris). Les
  22 relevés / 21 entrées de la garde égalent-ils ce que tu recomptes ? Chaque entrée est-elle justifiée par la ligne lue ?
- **A4 — la garde** (`local-date-guard.test.ts`) : l'auditeur modifié (`lignes?`) affaiblit-il la garde des **anciens** motifs
  (`ALLOWLIST`, `ALLOWLIST_SPECS`) ? Les sondes prouvent-elles quelque chose (une assertion dont les deux côtés sortent du même extracteur
  est vraie par construction) ? Angles morts écrits ? Rejoue mentalement la mutation M4 (journal `mut-M4.txt`).
- **A5 — tests** : chaque test neuf rougit-il sans le correctif (journaux `rouge-*.txt`) et pour la bonne raison ? Les valeurs d'ICU
  (`'25 nov. 2026, 00:30'`…) dépendent-elles de la version de Node ? Les `describe` à `Europe/Zurich` sont des filets (AC9), déclarés comme tels.
  Le piège du `$derived` (composant monté après `pinTimeZone`) est-il évité ? L'E2E (`tests/e2e/audit-log.spec.ts`, bloc `New York`) :
  attendu calculé dans le navigateur (`page.evaluate`), `timezoneId` par `test.use`, indépendance vis-à-vis des autres tests de la spec.
- **A6 — manuel, PDF compris** : `docs/manual/fr/user-manual.tex` (phrases modifiées vers `:790-796` et `:4690-4697`) et `admin-manual.tex` :
  la phrase nouvelle est-elle **exacte contre le code** (exception, limite) ? Aplatis avant de greper, `.tex` **et** PDF :
  `tr '\n' ' ' < f.tex | tr -s ' '` et `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '` (la ligature « ﬀ » casse « affiche » : cherche
  `heure de votre poste`). Les PDF ne sont pas régénérés dans la story (règle des fichiers communs) : le PDF dit encore l'ancienne
  phrase, c'est attendu ; vérifie que **le `.tex`** ne dit plus l'ancienne et que rien d'autre dans les manuels (`poste`, `fuseau`,
  `navigateur`, `UTC`, `Zurich`, `heure`) n'est rendu faux. Le manuel d'administration (§ sauvegarde automatique, § envois et prompts du
  serveur MCP) nomme-t-il un fuseau que la story change ? Les langues `de`, `en`, `it` : un `README.md` chacune, vérifie.
- **A7 — recette visuelle (AC13)** : les captures couvrent-elles chaque état touché (journal, détail d'import, section et liste des
  sauvegardes, bandeau, liste et détail des envois, détail d'un prompt), avant / après à New York, après à Zurich ? **Lis les images** :
  l'heure affichée « après » à New York égale-t-elle celle « après » à Zurich, et diffère-t-elle d'« avant » ? Thème clair.
- **A8 — périmètre et propagation** : `git diff --stat e5654fc00..HEAD -- frontend/src ':!*.test.ts'` ne nomme que `reports.api.ts`, les
  quatre sites et le commentaire de `mcp-prompts-format.ts` ; aucun fichier de `crates/` ; aucune clé i18n neuve (les quatre `.ftl`).
  **Greper le symptôme** (`heure locale`, `fuseau du navigateur`, `heure du poste`, `à l'heure locale`, `DateTimeFormat(` sans `timeZone`)
  dans `frontend/src`, `docs/`, `website/`, `README.md`, `CHANGELOG.md` : tout commentaire ou doc rendu faux est-il traité, ou laissé et
  dit (exception de D0, limite I2) ? Le CHANGELOG ne se touche pas dans une story (assemblé à l'intégration depuis les *Notes de version*).
- **A9 — règles du CLAUDE.md** : DRY (une seule lecture « naïf = UTC » dans `reports.api.ts` ; copies restantes `mcp-time.ts` et
  `settings/api-keys/+page.svelte:70-72`, nommées I2 et I1) ; documentation des API publiques ; `refs #1511` sans mot-clé de fermeture dans
  chaque message de commit (`git log e5654fc00..HEAD --format=%B | grep -niE '\b(close[sd]?|fix(e[sd])?|resolve[sd]?) #[0-9]'` ; attention :
  le **type** de commit `fix(frontend)` n'est pas un mot-clé suivi de `#`) ; modules touchés (règle de splitting) : reports,
  reconciliation, admin-backup, mcp-sends (+ la route audit-log, + un commentaire de mcp-prompts).
- **A10 — gates et comptes rendus** : le Dev Agent Record n'affirme-t-il que ce qui a tourné, chaque chiffre avec son journal et son
  commit (`front-gate.log`, `testfast.log`, `e2e-complet.log`) ? Les échecs E2E sont-ils jugés **fichier par fichier** contre
  `docs/testing.md` § *Les échecs attendus* ? Recompte les totaux depuis les journaux.

## Interdits

Lecture seule. N'écris **aucun** fichier du worktree ni du dépôt principal ; aucun `Edit`/`Write` hors de ton rapport. Aucun `cargo`,
`npm`, `npx`, `node`, `make`, `docker`, `mariadb`, `sqlx`, `curl` vers un serveur, aucun `scripts/*` (en particulier ni
`scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/gate-ci.sh`, ni `scripts/mem-guard.sh`, ni `scripts/recette-montee.sh`,
ni `scripts/install-hooks.sh`), aucun lancement de Playwright ni de Vitest. Aucun `git commit`, `git push`, `git checkout`, `git switch`,
`git stash`, `git merge`, `git restore`, `git worktree`. Aucun `gh` qui écrit (`issue create/edit/comment/close`, `pr …`, `api` avec
`-X`/`-f`), et **au moins une minute entre deux requêtes `gh`** (trois au plus). Rien sous `~/kesh/`. `python3 -I` en lecture pour
recompter est admis ; `pdftotext` admis.

## Rapport

Écris **un seul fichier** : `/home/gcorbaz/devel/kesh-gate-logs/e506/review-p1.md`, avec :

- le **verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW ;
- chaque finding : identifiant (`R1-n`), sévérité, axe, `chemin:ligne` et la ligne copiée qui le fonde, ce qui est faux, la correction
  proposée. Tout finding CRITICAL ou HIGH qui affirme l'absence d'un code attendu se vérifie d'abord par `grep -nF` (garde-fou du
  CLAUDE.md) ;
- la liste des axes **exercés** (ce que tu as relu, commande lancée) et **non exercés** (avec la raison) — un « 0 finding » sans cette
  liste ne compte pas comme passe ;
- les prémisses de la fiche confirmées au code (une ligne chacune, ligne citée).

Rends ensuite, en réponse, le verdict chiffré et le chemin du rapport.
