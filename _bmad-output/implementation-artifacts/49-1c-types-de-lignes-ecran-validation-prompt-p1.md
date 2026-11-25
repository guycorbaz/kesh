# Validation P1 de la fiche 49-1c-types-de-lignes-ecran — prompt de la passe

*Passe complète, contexte frais, **lecture seule**. Modèle : un autre que celui de la rédaction (la fiche est d'Opus 5.5 :
lancer cette passe sur **Sonnet**) ; jamais Haiku pour une première passe (CLAUDE.md § *Review Iteration Rule*).*

Tu valides la fiche de la story **49-1c-types-de-lignes-ecran** de Kesh (release 0.20.0, Epic 49, #1242, #1400, #1401).
Français soigné. Tu ne corriges rien : tu rends des findings.

- Worktree : `/home/gcorbaz/devel/kesh-49-1c`, branche `story/49-1c-types-de-lignes-ecran`.
- Fiche : `_bmad-output/implementation-artifacts/49-1c-types-de-lignes-ecran.md`. Code de référence : **`5156f2b7b`** (la
  tête avant le commit de la fiche) ; le code n'a pas bougé depuis (seule la fiche, `sprint-status.yaml` et ce prompt ont
  été commités).
- Rapport à écrire : `/home/gcorbaz/devel/kesh-gate-logs/e491c/validation-49-1c-p1.md`.

## À lire d'abord

1. `CLAUDE.md` du worktree (règles : *Review Iteration Rule*, *Inventorier les sites NON RÉSOLUS*, *Recette visuelle*,
   *Test Locally First*, *Migration breaking policy*, *Priorités des défauts*) ;
   `/home/gcorbaz/devel/kesh-gate-logs/prompts/e0200-commun.md` et `e0200-fiche.md` (ce que la fiche doit contenir).
2. `_bmad-output/planning-artifacts/epic-49-types-de-lignes-facture.md` (§ 49-1, § Contraintes de production, tableau des
   fichiers) et `/home/gcorbaz/devel/kesh-gate-logs/e491/decoupage-49-1.md` (§ 3, § 3 bis).
3. Les fiches closes `49-1a-types-de-lignes-migration-api.md` et `49-1b-types-de-lignes-pdf.md` (décisions, dettes
   renvoyées à la 49-1c : cherche « 49-1c » dans chacune).
4. Les issues, en lecture : `gh issue view <N> --json number,title,labels,milestone,body,comments` pour 1242, 1400, 1401,
   1364, 1363, 1110, 1374.

## Axes — chacun à déclarer exercé ou non exercé

1. **Périmètre contre le fichier d'épique** : chaque élément de la 49-1c (sélecteur de type, ligne de commentaire, avoir,
   ouverture de `comment`, `heading` fermé, `verify_vat_rates_against_db`, édition qui renvoie le type, i18n ×4, Vitest,
   E2E, manuel utilisateur FR) a son AC et sa tâche ; les écarts déclarés (D4 `vat.rs` inchangé, D11 documents en plus,
   `kesh-db` en plus, `ProductPicker` inchangé) sont fondés ; rien n'est pris à la 49-2, 49-3, 49-4 ou 49-1d.
2. **Prémisses relues au code** : chaque citation `chemin:ligne` de § *Ce qui se lit au code* et de § *Inventaire* se relit
   à `5156f2b7b` (`git show 5156f2b7b:<chemin> | sed -n '<l>p'`) ; une ligne fausse ou un fait faux est un finding.
   Vérifie en particulier : la garde vit dans `kesh-db` ; `.nfc()` à `invoices.rs:507` s'applique **avant** la bifurcation
   par type ; `verify_vat_rates_against_db` a exactement deux appelants de lignes ; l'avoir reprend le type (49-1a).
3. **Inventaire par le symptôme** : rejoue les commandes S1 à S4 ; cherche, **en partant du symptôme et non de la liste**,
   d'autres sites : un code qui suppose qu'une ligne de facture client est chiffrée (écran, API, services, MCP, exports,
   courriel, rappels, écran de l'avoir, création d'avoir), un texte (code, doc-comment, i18n ×4, manuels, `docs/`) qui
   dit `comment` refusé, un écran qui lit `invoice.lines` ou `creditNote.lines`. Tout site trouvé hors de la fiche, sans
   issue, est un finding.
4. **Critères d'acceptation constatables** : chaque AC se constate par une commande, un test ou une capture nommés ; un AC
   flou ou invérifiable est un finding.
5. **Preuve du rouge** : chaque test neuf (T1 à T10, T12) a une mutation qui le fait rougir **sans** le correctif et que
   le test, tel que décrit, tuerait réellement (en particulier : T8 joue-t-il vraiment `reloadFromServer` ? T4 désactive-t-il
   vraiment le 0 % du seed ? T5 est-il rouge sans `.nfc()` ?). Une mutation qui survivrait est un finding.
6. **Factures réelles en production** : AC8 prouve-t-il que les factures et avoirs existants restent identiques (lignes,
   totaux, TVA, écritures, numéros, PDF, écran, `PUT` d'un brouillon chiffré) ? Un chemin non couvert (PDF non figé,
   regel, avoir rendu à la demande, brouillon réenregistré) est un finding.
7. **Décisions** : chaque D a son option écartée et son motif ; D9 (#1401 par la preuve) et D10 (#1400 hors du code) sont
   jugés **sur le fond** — la prémisse de D9 tient-elle au code ? le motif de D10 (« une issue, une release », rendu des
   factures existantes) tient-il ? D12 (pas de borne de longueur) est-il cohérent avec la 49-1b (D6) et #1364 ?
8. **Manuel, PDF compris** : `docs/manual/fr/user-manual.tex` (`:1456`, `:1594`, `:1924`), `admin-manual.tex:2559`,
   `docs/api-external.md:512` ; aplatis aussi les PDF (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`,
   idem `admin-manual.pdf`) et cherches-y ce que la story rendra faux ou devra dire (lignes de facture, commentaire,
   avoir, caractères imprimés, capacité du PDF). Un texte qui deviendra faux sans tâche dans la fiche est un finding.
9. **E2E et recette visuelle** : le parcours dans le navigateur est exigé (T10) ; les captures couvrent chaque état touché,
   sur une base de démonstration ; les gates sont décrits selon le `CLAUDE.md` (ciblé pendant la boucle, `gate-ci` et E2E
   complet au dernier commit de code, bases `kesh_e491c`, `kesh_ci_e491c`, `kesh_e2e_e491c`).
10. **Splitting et migration** : le compte des modules (cinq au grain de la crate) est-il juste ? La story n'exige-t-elle
    vraiment aucune migration ?
11. **Coordination** : fichiers communs avec la 49-1d (manuel d'administration), la 48-14 (catalogues), la 49-2 et la 49-3
    (`InvoiceForm.svelte`), la PR #1327 de Beat.

## Interdits

Lecture seule. **Aucune écriture** hors de ton rapport (`/home/gcorbaz/devel/kesh-gate-logs/e491c/validation-49-1c-p1.md`).
Aucun `cargo`, `npm`, `npx`, `make`, `docker`, `mariadb`, `sqlx` ; aucun script de `scripts/` (ni `scripts/prepare-release.sh`,
ni `scripts/test-fast.sh`, ni `scripts/gate-ci.sh`, ni `scripts/recette-montee.sh`, ni aucun script mutant) ; aucun
`git commit`, `git checkout`, `git switch`, `git reset`, `git stash`, `git push`, `git merge`, `git worktree` ; aucun `gh` qui
écrit (`gh issue create|edit|comment|close`, `gh pr …`). Rien sous `~/kesh/` (production). Aucune donnée réelle de Guy.
Ne touche à aucun autre worktree (`kesh-49-1d`, `kesh-48-14` travaillent en parallèle).

## Rapport

En tête : **verdict chiffré** `CRITICAL / HIGH / MEDIUM / LOW` (par exemple `0/0/2/5`). Puis chaque finding : identifiant
(`V1-n`), sévérité, axe, constat **avec la ligne lue citée** (`chemin:ligne` et la ligne copiée, ou la commande et sa
sortie), conséquence, remède proposé. Puis la liste des **axes exercés** (ce que tu as fait pour chacun : commandes
lancées, fichiers lus) et des **axes non exercés** (avec la raison). Un « 0 finding » sans cette liste ne compte pas comme
passe. Pour tout finding CRITICAL ou HIGH qui affirme l'absence d'un code ou la présence d'un anti-pattern, cite le
`grep -nF` qui l'établit.
