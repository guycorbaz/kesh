# Validation P1 de la fiche 49-4 — prompt (passe complète, contexte frais)

Tu valides la fiche **`/home/gcorbaz/devel/kesh-49-4/_bmad-output/implementation-artifacts/49-4-designation-longue-pdf.md`**
(story 49-4 de Kesh, release 0.20.0, Epic 49 ; issues #1363, #1364, #1400). Passe **complète**, **lecture seule**, en français.
Modèle attendu : **Sonnet** (la fiche a été rédigée par Opus). Tu ne connais de la story que ce que tu lis : relis tout au code.

## À lire

- `/home/gcorbaz/devel/kesh-49-4/CLAUDE.md` (règles ; § *Review Iteration Rule*, § *Inventorier les sites NON RÉSOLUS*,
  § *Recette visuelle*, § *Migration breaking policy*).
- `/home/gcorbaz/devel/kesh-gate-logs/prompts/e0200-commun.md` et `e0200-fiche.md` (consignes de la release et de la fiche).
- `_bmad-output/planning-artifacts/epic-49-types-de-lignes-facture.md` (ligne 49-4 du tableau des stories, règles de sortie,
  § *Contraintes de production*) et `release-0.20.0.md`, dans le worktree.
- La fiche close de la 49-1b (`_bmad-output/implementation-artifacts/49-1b-types-de-lignes-pdf.md`, D4 à D8) ; la fiche de la
  49-1c : `git -C /home/gcorbaz/devel/kesh-49-1c show story/49-1c-types-de-lignes-ecran:_bmad-output/implementation-artifacts/49-1c-types-de-lignes-ecran.md`
  (D10, D12, AC10, Dev Notes « La 49-4 ») ; la fiche de la 49-1d :
  `git -C /home/gcorbaz/devel/kesh-49-1d show HEAD:_bmad-output/implementation-artifacts/49-1d-types-de-lignes-recette-montee.md` (D8).
- Les issues : `gh issue view <N> --json number,title,body,comments` pour 1363, 1364, 1400, 1110 (lecture seule).
- Le code à la tête du worktree `/home/gcorbaz/devel/kesh-49-4` (`git -C … rev-parse HEAD` ; la fiche cite `3379a971a` pour le
  code) : `crates/kesh-qrbill/src/{pdf,texte,largeurs,types,lib}.rs`, `crates/kesh-qrbill/tests/rendu_avant_apres.rs`,
  `crates/kesh-api/src/routes/{invoices,invoice_pdf_service,issued_invoice_pdf,credit_notes}.rs`, `crates/kesh-api/src/errors.rs`,
  `crates/kesh-api/tests/pdf_avant_apres_e2e.rs`, `crates/kesh-db/src/repositories/invoices.rs` (validation, dévalidation),
  `frontend/src/routes/(app)/invoices/[id]/+page.svelte`, `docs/api-external.md`, `docs/manual/fr/{user,admin}-manual.tex`.
- La PR #1327 : `gh pr diff 1327` (lecture seule).

## Axes (tous à exercer ; dis pour chacun s'il l'a été)

1. **Périmètre contre le fichier d'épique** : la fiche couvre-t-elle toute la ligne 49-4 (#1363, #1364, #1400, reprise des
   phrases du manuel posées par la 49-1c, règle « la 49-1c ne sort pas sans la 49-4 ») et rien d'autre ? Écarts déclarés ?
2. **Prémisses relues au code** : chaque citation `chemin:ligne` de « Ce qui se lit au code » et des décisions, copiée et
   relue ; en particulier : l'ordonnée constante du tableau (`pdf.rs:548`), la garde (`:653`, `:665`), la troncature (`:668`),
   l'absence de contrôle à la validation (`routes/invoices.rs:931-944`), l'affichage du message à l'écran (`+page.svelte:448`),
   l'avoir et le rappel jamais figés, la dévalidation.
3. **Faisabilité des décisions** : D4 (contrôle pur, sans contact ni banque, même plan que le dessin ; arrondi calculé comme à la
   validation ; course assumée) ; D2 (repli sur le rendu de la 0.19.0 : atteignable seulement pour une pièce validée sans le
   contrôle ; conserve-t-il l'octet ?) ; D6 (source unique entre `caracteres_sans_glyphe` et le filtre du rendu ; listes closes).
   Un défaut d'une décision qui rendrait une facture **existante** impossible à imprimer, ou qui changerait son rendu hors de la
   liste de D3, est au moins **HIGH**.
4. **Inventaire par le symptôme** : les trois symptômes (S1 troncature « … », S2 refus après la validation, S3 caractère perdu) ;
   rejoue les commandes, cherche un site **omis** par un autre chemin (ne pars pas de la liste de la fiche) ; chaque site est-il
   dans le périmètre ou hors périmètre avec une issue (existante, vérifiée par `gh issue view`, ou « à ouvrir ») ?
5. **AC constatables** : chacun se vérifie-t-il par une commande, un test ou une capture nommés ? Renvois AC ↔ tests ↔ décisions
   complets ?
6. **Preuve du rouge** : chaque test neuf a-t-il une mutation qui le fait rougir **pour la bonne raison** (pas une erreur de
   compilation, pas un `assert status == 200` seul) ? T-SRC et T-PROP prouvent-ils ce qu'ils disent, ou sont-ils vrais par
   construction (`CLAUDE.md` : « une assertion dont les deux côtés sortent du même extracteur ne prouve rien ») ?
7. **Factures réelles en production** : la dérogation (D3) est-elle écrite explicitement, avec son périmètre exact (jamais
   téléchargée, regel, avoir, rappel), ce qui change et ce qui ne change pas ? Le PDF archivé reste-t-il servi tel quel ? Les
   tests avant / après de la 49-1b (fixtures `avant-0.20.0`) restent-ils la référence, sans modification sauf S7 ? Cohérence
   avec la D8 de la 49-1d (« 44 caractères au plus ») : la fiche la signale-t-elle correctement ?
8. **Manuel, PDF compris** : `git grep` des phrases touchées dans `docs/manual/fr/*.tex`, et le PDF du manuel s'il existe
   localement (`docs/manual/fr/*.pdf`, non commité ; sinon le dire) aplati : `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`, grep
   de « sans avertissement », « trait d'union », « huit », « 44 ». Les phrases de la 49-1c à reprendre sont-elles toutes nommées,
   et l'ordre de fusion (D9) tenable ?
9. **Coordination** : recoupements avec la PR #1327 (relis son diff) et la 48-11 (`epic-48-corrections-0-20-0.md`) exacts ?
   Fichiers communs avec 49-1c, 49-1d, 49-3 dits ?
10. **Splitting et charge** : compte des modules (au grain de la crate) ; la story tient-elle à la coupure du jeudi 2026-11-26 ;
    le découpage proposé (49-4a / 49-4b) est-il cohérent avec la règle de sortie et la règle « une issue, une release » ?
11. **Migration** : la story n'en exige aucune (D10) — vérifie qu'aucune décision n'en suppose une.
12. **Forme** : en-tête, sections imposées par `e0200-fiche.md`, Note de version sans numéro d'épique, aucun mot-clé de fermeture,
    aucune donnée réelle.

## Interdits

Lecture seule. N'écris **aucun** fichier hors de ton rapport. Aucun `cargo`, `npm`, `npx`, `make`, `docker`, `mariadb`,
`sqlx`, aucun script de `scripts/` (en particulier `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`,
`scripts/recette-montee.sh`, `scripts/mem-guard.sh`), aucun `git` qui écrit (`commit`, `checkout`, `switch`, `merge`, `rebase`,
`reset`, `stash`, `worktree add/remove`, `push`), aucun `gh` qui écrit (`issue create/edit/comment`, `pr …` autre que `view` et
`diff`). Rien sous `~/kesh/`. Ne touche à aucun worktree.

## Rapport

Écris ton rapport dans **`/home/gcorbaz/devel/kesh-gate-logs/e494/validation-49-4-p1.md`** :

- verdict chiffré `CRITICAL/HIGH/MEDIUM/LOW` (ex. `0/1/3/5`) en tête ;
- chaque finding : identifiant (V1-n), sévérité, lieu dans la fiche, **preuve** (commande et sortie, ou ligne de code copiée),
  remède proposé ; un finding CRITICAL ou HIGH qui affirme une absence se prouve par `grep -nF` ;
- la liste des **axes exercés et non exercés** (1 à 12), avec, pour un axe non exercé, la raison. Un « 0 finding » sans cette liste
  ne compte pas.
