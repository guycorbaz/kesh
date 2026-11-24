# Revue de code P2 de la story 49-4a — passe complète (Sonnet, contexte frais, un seul agent)

**Un agent Sonnet**, contexte frais, qui exerce **l'une après l'autre** les trois lentilles **B** (Blind Hunter), **E** (Edge Case
Hunter) et **A** (Acceptance Auditor) sur **toute la story et sur la remédiation de la P1**. Passe **complète**, non ciblée (la
remédiation touche un catalogue i18n et un module de test de `kesh-qrbill` : code de production au sens de la règle). Français
soigné. Aucune question : tu tranches et tu le dis.

## Cadre

- Worktree : `/home/gcorbaz/devel/kesh-49-4`, branche `story/49-4-designation-longue-pdf` (porte aussi la fiche 49-4b, **hors
  revue**). Fiche : `_bmad-output/implementation-artifacts/49-4a-capacite-pdf-avant-validation.md` — elle fait foi (D1 à D7,
  AC1 à AC11, tests T1 à T6 et T10, Dev Agent Record, Change Log).
- **Base du diff** : `87e00ba05` (`git merge-base HEAD origin/epic/49-types-de-lignes-facture`). Diff complet :
  `git -C /home/gcorbaz/devel/kesh-49-4 diff 87e00ba05..HEAD`. Le commit de fusion `b1d9d4776` se relit en diff combiné
  (`git show --cc b1d9d4776`).
- **Remédiation de la P1** : `git log --oneline fa7ff0901..HEAD` et `git diff fa7ff0901..HEAD`. Les trois rapports de la P1 sont
  dans `/home/gcorbaz/devel/kesh-gate-logs/e494/review-p1-{B,E,A}.md` (Opus : 0/0/1/9). Le Change Log de la fiche dit le sort de
  chaque finding ; **vérifie chaque sort au code** (finding corrigé : l'est-il ? finding écarté ou renvoyé en issue : la
  raison tient-elle ?).
- **Ce qui a tourné** : Dev Agent Record, § *Réalisation* et § *Revue P1 remédiée* ; journaux dans
  `/home/gcorbaz/devel/kesh-gate-logs/e494/dev/` (dont `remed-*.log`). **Non exercé** : E2E Playwright complet (clôture, après la
  revue).
- Kesh est **en production** chez Guy, avec de vraies factures émises et remises sur papier. **Vérifie qu'aucune facture émise
  ne voit son PDF changer** : aucun octet, ni pour une facture validée, ni pour un avoir, un rappel ou un PDF figé. Ce point est
  le premier du verdict, avec la preuve (diff de `pdf.rs` contre `87e00ba05` opération par opération, tests avant/après non
  modifiés et verts au journal, chemins de `issued_invoice_pdf.rs` et du rendu des factures validées inchangés).
- Décisions de Guy du 2026-11-20 : **aucune réserve du bloc de rappel** (D2 ; limite au manuel, sortie par avoir, plusieurs
  pages en v0.21.0, #1374) ; ne rien élargir : un point hors fiche s'écrit comme issue à ouvrir.

## Lentilles (toutes trois, sur toute la story)

- **B — Blind Hunter** : le diff seul d'abord (`git diff 87e00ba05..HEAD -- crates frontend/tests`), sans la fiche, puis la fiche
  pour trier. Code, logique, tests qui prouvent moins qu'ils ne disent (vrais par construction, oracle partagé, montage qui
  passe à vide). Relis en particulier la famille engendrée de T2 après la remédiation (désignations chiffrées longues ajoutées :
  les seuils `acceptees > 100 && refusees > 100` tiennent-ils, la famille reste-t-elle déterministe ?) et le test neuf de T1
  (409 et 404).
- **E — Edge Case Hunter** : chaque branche et chaque bord — seuils à l'égalité flottante (`ty < plancher`), récapitulatif de
  0 à 5 lignes, arrondi nul au centime (`0.004`) ou non (`0.005`), commentaires vides, blancs, sans rien d'imprimable
  (« Ω », U+200B), pré-filtre après le contrôle, brouillon absent ou non brouillon (404/409), revalidation d'une facture
  dévalidée, réglage `round_to_5_centimes` absent, avoir et rappel non contrôlés, course `PUT`/validation (assumée, D1).
- **A — Acceptance Auditor** : chaque AC (AC1 à AC11) et chaque décision D1 à D7 contre le code, les tests et le Dev Agent
  Record ; chaque chiffre du Dev Agent Record contre son journal (chiffre, commit, périmètre) ; la fiche à jour (tâches
  cochées selon ce qui est fait, préambule, chemins).

## Axes obligatoires

1. **Aucun octet de PDF ne change** (voir Cadre) : `crates/kesh-qrbill/src/pdf.rs` (géométrie extraite : mêmes opérations `f32`,
   même ordre) ; `git diff 87e00ba05..HEAD --stat -- crates/kesh-qrbill/tests crates/kesh-api/tests/fixtures
   crates/kesh-api/tests/pdf_avant_apres_e2e.rs crates/kesh-qrbill/src/empreinte.rs` vide.
2. **Source unique, oracle indépendant** (D1, AC3).
3. **La route** : `crates/kesh-api/src/routes/invoices.rs` (`validate_invoice_handler`) et
   `crates/kesh-api/src/routes/invoice_pdf_service.rs` : rien d'écrit sur un refus, arrondi, ordre contrôle puis pré-filtre.
4. **Erreur et textes** : `crates/kesh-api/src/errors.rs`, les quatre catalogues
   `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` (clé `error-invoice-lines-do-not-fit-pdf` ; la
   remédiation a corrigé l'allemand : « ihres PDFs: sie benötigen »), `crates/kesh-api/tests/textes_coherents.rs`. Grammaire des
   quatre langues relue, vouvoiement cohérent avec les messages voisins.
5. **Tests** : `crates/kesh-api/tests/invoice_validation_pdf_e2e.rs` (T1, quatre tests), `pdf.rs::tests`,
   `frontend/tests/e2e/invoice-validation-pdf.spec.ts` (T6), leurs rouges au journal.
6. **Le manuel, PDF compris, et le guide de l'API** (`CLAUDE.md` § *Le prompt d'une passe doit NOMMER le manuel*) :
   `docs/manual/fr/user-manual.tex` (item « Lignes de facture », `:1477` : la remédiation a rétabli « huit lignes **imprimées**
   au plus », « sans numéro neuf », « raccourcissez les commentaires »), `docs/manual/fr/admin-manual.tex` (`sec:maj-0-20-0`),
   `docs/api-external.md` (§ *Valider une facture — des lignes qui ne tiennent pas sur la page du PDF* `:526`, table des codes
   `:1313-1314`). Le texte du manuel dit-il ce que fait le code, et s'accorde-t-il mot pour mot en substance avec
   `api-external.md` ? La résolution du conflit de fusion `b1d9d4776` n'a-t-elle fait perdre aucune autre phrase de la 49-1c
   (`git diff 87e00ba05 b1d9d4776 -- docs crates/kesh-i18n` ne doit montrer que des ajouts de la 49-4a et le remplacement
   voulu) ? Les PDF du dépôt (`docs/manual/fr/*.pdf`) ne sont pas régénérés (jamais commités dans `story/…`) et ne prouvent rien.
   Le PDF compilé **à ce HEAD** est `/home/gcorbaz/devel/kesh-gate-logs/e494/manuels/user-manual.pdf` ; **forme permise**,
   aplati : `pdftotext /home/gcorbaz/devel/kesh-gate-logs/e494/manuels/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -o 'tient sur une page.\{0,700\}'`
   (le PDF emploie l'apostrophe typographique ; ne grepe pas `d'une`). Le manuel d'administration est inchangé par la
   remédiation : `/home/gcorbaz/devel/kesh-gate-logs/e494/manuels/admin-manual.pdf`, fragment `LINES_DO_NOT` ou `INVOICE_LINES`.
   Les manuels DE/IT/EN : existent-ils, portent-ils une phrase de capacité ? (à déclarer exercé ou non).
7. **Captures** (AC7) dans `/home/gcorbaz/devel/kesh-gate-logs/e494/captures/` : les regarder (outil de lecture d'image) et les
   comparer au code (8 lignes sans récapitulatif, 6 avec TVA unique, aucun chevauchement au seuil).
8. **Périmètre et coordination** : `bash /home/gcorbaz/devel/kesh-gate-logs/e494/dev/controles-diff.sh /home/gcorbaz/devel/kesh-49-4 87e00ba05`
   (lecture seule, sortie vide attendue) ; `kesh-qrbill/src/lib.rs` ne change que par le ré-export ; aucune ligne de `kesh-db`,
   `kesh-core`, `frontend/src`, `CHANGELOG.md`.
9. **Propagation post-patch** (`CLAUDE.md` § *Propagation post-patch — grep du symptôme*) : pour chacun des correctifs de la
   P1, le symptôme a-t-il été grepé sur tout le dépôt (manuels DE/IT/EN, fiches, doc-comments, catalogues) et tous les sites
   traités ? Refais-en au moins deux toi-même (`git grep -nF` du jeton, pas de la formulation).

## Interdits

Lecture seule. **Aucune** commande qui écrit, compile ou exécute : ni `cargo` (build, test, nextest, clippy, fmt), ni `npm`,
ni `npx playwright`, ni `make`, ni `xelatex`, ni `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/prepare-release.sh`,
`scripts/mem-guard.sh`, ni tout script mutant, ni `sqlx`, ni `docker`, ni `git` qui écrit (`commit`, `checkout`, `switch`,
`reset`, `restore`, `stash`, `push`, `tag`, `rebase`, `merge`, `worktree`), ni `gh` en écriture, ni `rm`, `mv`, `cp`,
`sed -i`, `perl -pi`, ni redirection `>`/`>>`, ni fichier temporaire. Permis : `git diff`, `git show`, `git log`, `git grep`,
`git cat-file`, `cat`, `sed -n`, `/usr/bin/grep`, `ls -l`, `pdftotext … -` (forme de l'axe 6), `controles-diff.sh` et l'outil
de lecture d'image. `grep` est une fonction vers `ugrep` : utilise `/usr/bin/grep` ou `git grep`. Rien sous `~/kesh/`.

## Rapport

Seul fichier que tu écris, par l'outil d'écriture : `/home/gcorbaz/devel/kesh-gate-logs/e494/review-p2.md`.

- Verdict `CRITICAL/HIGH/MEDIUM/LOW = n/n/n/n`, et en tête : **« aucune facture émise ne voit son PDF changer »** vrai ou faux,
  preuve citée.
- Chaque finding : sévérité, lieu (`fichier:ligne`), **prémisse relue** (toute affirmation de présence ou d'absence = sortie
  `git grep -nF` ou `/usr/bin/grep -nF` copiée), conséquence, remède proposé, et s'il touche la production. Un finding CRITICAL
  ou HIGH qui affirme une absence se vérifie lui-même par un `grep -nF` avant d'être rendu.
- **Axes exercés et axes non exercés**, listés par lentille (B, E, A) : un « 0 finding » sans cette liste ne compte pas.
- Le sort que tu donnes à chaque finding de la P1 (corrigé vérifié / écarté justifié / issue), une ligne chacun.
