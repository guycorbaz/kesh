# Prompt — revue de code P1 de la story `1582-nom-comptabilite` (#1582), passe complète

Tu relis, **en lecture seule**, le code de la story `1582-nom-comptabilite` de Kesh (frontend seul, aucune migration, hors epic, visée v0.20.0).
Fiche (fait foi, y compris les décisions du Project Lead : Q1 groupes repliés, D1 `company.name`, D5 « Kesh — <nom> — <page> », D6 `MutationObserver`) :
`/home/gcorbaz/devel/kesh-1582/_bmad-output/implementation-artifacts/1582-nom-comptabilite.md`.

- **Passe complète, première de la boucle. Modèle : Opus. Contexte frais.** Langue du rapport : français.
- **Entrée** : le diff **aplati** de la branche contre `main`, hors fiche et prompts :
  `git diff origin/main...story/1582-nom-comptabilite -- frontend docs/manual/fr/user-manual.tex` (depuis `/home/gcorbaz/devel/kesh-1582`).
  Fichiers de production : `frontend/src/lib/app/stores/company-identity.svelte.ts`, `frontend/src/lib/shared/utils/tab-title.ts`,
  `frontend/src/lib/shared/components/AppBrand.svelte`, `frontend/src/routes/(app)/+layout.svelte`, `frontend/src/routes/(app)/settings/+page.svelte`.
  Tests : `tab-title.test.ts`, `company-identity.svelte.test.ts`, `AppBrand.test.ts`, `settings-page.test.ts` (un test ajouté), `frontend/tests/e2e/company-name-header.spec.ts`.
- **Captures jointes** (thème clair, base de démonstration jetable) : `/home/gcorbaz/devel/kesh-gate-logs/s1582/captures/` — ouvre-les (lecture d'image) et dis ce que tu y vois :
  `1-societe-*`, `2-personne-physique-*`, `3-nom-long-survol-*`, `4a-groupes-deplies-*`, `4b-groupes-replies-*`, `5-onglets.png`, `6-demonstration-*`, `7-lecture-en-echec-*` ; mesures : `mesures.txt`.
  Rappels : une capture sans interface n'a ni barre d'onglets ni infobulle native (la 5 est reconstituée à partir des titres réels ; la 3 prouve le survol par `title`, non l'infobulle).
- Journaux : mutations `/home/gcorbaz/devel/kesh-gate-logs/s1582/dev/mutations.md` ; gate frontend `…/dev/gate-front.log` ; E2E `…/e2e/e2e-full.log`.

## Garde-fou `grep -nF`

Pour tout finding CRITICAL ou HIGH qui affirme l'absence d'un code attendu ou la présence d'un anti-pattern, donne la commande `grep -nF "<chaîne>" <fichier>`
(`-F` obligatoire) et sa sortie copiée ; sans cette preuve, le finding est écarté. Cite toujours `chemin:ligne`.

## Interdits

Tu n'écris **aucun fichier** hors de ton rapport. Tu ne lances **ni** `cargo`, **ni** `npm` / `npx` (dont `vitest`, `playwright`), **ni** `make`, **ni** aucun `scripts/*`
(dont `scripts/prepare-release.sh`, `scripts/mem-guard.sh`), **ni** `docker`, `mariadb`, `sqlx`, `curl`, `wget`, **ni** `git commit`, `checkout`, `stash`, `restore`,
`reset`, `merge`, `rebase`, `fetch`, `push`, `worktree`, **ni** `gh` en écriture, **ni** `sed -i`, `perl -pi`, `tee`, `>`, `>>` ou toute redirection vers un fichier,
**ni** `pkill`, `killall`, `kill` par motif, **ni** `python3` ou autre interpréteur. Rien sous `~/kesh/`. Permis : `cat`, `grep`, `sed -n`, `ls`, `wc`, `find` (sans `-exec` ni `-delete`),
`git log`, `git show`, `git diff`, `git grep`, `pdftotext` (lecture), la lecture d'images, `gh … view|list`.

Rapport : **`/home/gcorbaz/devel/kesh-gate-logs/s1582/code-review-p1.md`** (le seul fichier que tu écris).

## Axes à exercer (chacun exercé ou déclaré non exercé, avec la raison)

1. **Acceptation AC1 à AC12** de la fiche, un par un, au code et aux tests (preuve : ligne lue).
2. **Réactivité Svelte 5** : le premier `$effect` du layout (`untrack`, `clear` au nettoyage), le second (`watchTabTitle`), `effect_update_depth_exceeded`, la séquence `let` du store ; le démontage par `/onboarding` (AC8).
3. **`tab-title.ts`** : la table de la fiche, l'idempotence, la garde anti-boucle, la restauration conditionnelle, les titres que la page réécrit elle-même (changement de langue), la course entre `apply()` initial et un titre de page posé après ; fuites d'observateur.
4. **`adoptCompany`** : `companyIdentity.set` seulement quand la société est adoptée ; tous les chemins qui y passent (nom, adresse, e-mail, contacts, devises, relecture après conflit) ; une réponse périmée ne l'écrit pas.
5. **Accessibilité et rendu** : nom accessible du lien « Kesh <nom> », `div` dans `<a>`, contraste, `truncate` / `max-w-56`, hauteur 56 px, aucune clé i18n nouvelle, les captures (alignement, déformation du champ de recherche, menu profil visible à 1280 px).
6. **Tests** : chaque test prouve-t-il ce qu'il dit (mutations M1 à M12 : relis le journal ; M3 a survécu avant renforcement ; M11a, M11b, M12 déclarées sans test) ? Un test vert par construction (deux côtés du même extracteur) ? L'E2E 5 (démonstration) cherche le bandeau par sa classe, faute de `data-testid` (DemoBanner inchangé) : est-ce acceptable ?
7. **Le manuel (`docs/manual/fr/user-manual.tex`) contre le code, PDF compris** : lis `docs/manual/fr/user-manual.pdf` aplati avant de greper (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) ; le PDF n'est pas régénéré dans la story (il se régénère à la release) : l'écart PDF/`.tex` est attendu, tout autre écart de ce que le manuel dit de l'en-tête, du nom et du titre de l'onglet est un finding. Les manuels DE/IT/EN n'ont que `README.md`.
8. **Périmètre** (AC12) : `git diff --stat origin/main -- crates/` vide ; aucun fichier de page, d'i18n, migration ; `DemoBanner` inchangé ; propagation du symptôme (le même nom lu ailleurs par un autre appel, un autre titre posé par le layout).
9. **Hors périmètre à ne pas rouvrir** : les décisions du Project Lead (D1, D5, D6, Q1) ; les 11 pages sans titre (issue P4 de la fiche) ; la composition « Prénom Nom » côté serveur (issue `technical-debt`).

## Rapport

Pour chaque finding : `R1-n`, sévérité (CRITICAL, HIGH, MEDIUM, LOW), constat avec la ligne lue citée, conséquence, remède. En fin de rapport : décompte
**CRITICAL / HIGH / MEDIUM / LOW** recompté depuis tes findings ; **axes exercés** et **non exercés** (avec la raison) ; les commandes lancées. Un « 0 finding » sans cette liste ne compte pas comme passe.
