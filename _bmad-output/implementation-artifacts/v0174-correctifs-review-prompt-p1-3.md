# Prompt — revue de code P1 de la fiche `v0174-correctifs-3` (les écrans : #888, #889, #872, #869, #817), trois lentilles, Sonnet

Tu relis le développement de la fiche **`v0174-correctifs-3`** de Kesh (comptabilité suisse, SvelteKit, Rust/Axum,
MariaDB 10.11), release **0.17.4**, **sans migration** :
- **#888** — le formulaire d'un lot de paiement lit toutes les pages des factures validées (200 par page), s'arrête sur
  une page vide, déduplique par `id` ;
- **#889** — `todayLocal` déménage dans `$lib/shared/utils/local-date` ; douze sites de la date du jour, le rappel
  manuel, neuf affichages d'instants, la liste des utilisateurs ; garde `local-date-guard.test.ts` (inventaire clos,
  huit sites conformes) ; *rollout* mécanique, **à relire fichier par fichier** ;
- **#872** — le formulaire d'une facture fournisseur rend les projets orphelins et marque « (archivé) » ;
- **#869** — le refus `PERIOD_LOCKED` du repassage en brouillon nomme la date du verrou ;
- **#817** — `ContactResponse.addressCountryKnown`, l'avis sous le champ *Pays*.
Trois clés i18n (quatre locales) ; trois gardes recomptées.

Trois lentilles, **une par rapport** : **B** (chasseur aveugle), **E** (cas limites), **A** (auditeur d'acceptation).

Worktree `/home/gcorbaz/devel/kesh-0174c`, branche `story/v0174-correctifs`. Le diff de cette fiche, **aplati** :
`git -C /home/gcorbaz/devel/kesh-0174c show 15742cad` (et tout commit de code de cette fiche qui le suivrait, au Change
Log). Langue : français. Modèle : **Sonnet**. Passe **complète** (première passe). Contexte frais.

Rapports (un par lentille, **le seul fichier que tu écris**) :
`/home/gcorbaz/devel/kesh-gate-logs/v0174c-review-p1-3-B.md`, `…-3-E.md`, `…-3-A.md`.

## À lire (lentilles E et A)

- La fiche `_bmad-output/implementation-artifacts/v0174-correctifs-3.md` (§ 3.1 à 3.5, AC-3.1 à 3.15, MC-888,
  MC-888-b, MC-889-a/b/c/g, MC-872, MC-869, MC-817, Dev Notes, **Dev Agent Record**, Change Log) ; l'index ; le registre
  `v0174-choix-autonomes-v0174c.md`, **C-v0174c-9, 12 à 19, 24 à 26**.
- Pièces hors dépôt (lecture seule) : `kesh-gate-logs/v0174c/rouges.log`, `rouges-888.log`, `rouges-fiche3.log`,
  `verts-fiche3.log`, `vitest-final.log`, `check-final.log`, `npm-build-final.log`, `mut/mutations.log`,
  `mut/MC-*.log`, `e2e-*.log` ; **les captures** `kesh-gate-logs/v0174c/captures/*.png` (ouvre-les : thème clair ;
  lot de plus de 200 factures, projet archivé sous racine archivée, refus nommant la date du verrou, avis du pays
  inconnu, refus `XX` aux deux écrans).
- Code : `frontend/src/lib/shared/utils/{local-date,local-date-guard.test,pin-clock.test-helper}.ts`, et chaque
  fichier du diff ; `crates/kesh-i18n/locales/*/messages.ftl` (trois clés) ; `frontend/tests/e2e/v0174-correctifs.spec.ts`.
- Manuel : `docs/manual/fr/user-manual.tex` (lots pain.001, *Modifier* un brouillon, repassage en brouillon, adresse
  d'un contact) **et** son PDF aplati.

## Interdits absolus

- **Lecture seule.** Aucun fichier hormis ton rapport, **aucun fichier temporaire**, **aucune redirection**.
- **Aucun `rm`** ; **aucun `git` qui écrit**.
- **Rien qui compile ou exécute** : ni `npm`/`npx`/`node`/`vitest`/`playwright`, ni `cargo`, ni `python`, ni `make`,
  ni `docker`, ni `curl`, ni `claude`, ni `sed -i`, ni **`scripts/prepare-release.sh`**, ni aucun script de
  `kesh-gate-logs`. **Aucune commande `gh`.**
- Permis : `git diff/show/log/grep`, `grep`, `sed -n`, `awk` (sans redirection), `cat`, `ls`, `find` (sans `-exec`),
  `wc`, `sort`, `uniq`, la lecture d'images, `timeout 60 pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' '` puis
  `grep -cF`/`grep -oF` à motifs fixes.
- Toute affirmation de présence ou d'absence : une sortie **`grep -nF`** copiée.
- **Données réelles** : aucune.

## Axes obligatoires, pour chaque lentille

1. **L'inventaire clos de #889.** Rejoue (lecture) les deux motifs de la garde sur l'arbre :
   `git grep -nE 'toISOString\(\)\.slice\(0, ?10\)' -- frontend/src ':!*.test.ts' ':!*.test-helper*'` et
   `git grep -nE "formatSwissDate\(.*\.slice\(0, ?10\)" -- frontend/src ':!*.test.ts'` : exactement les huit entrées de
   l'allowlist ? Formes **voisines** non relevées (`toJSON()`, `split('T')`, `substring(0, 10)`, `toLocaleDateString`,
   `new Date(iso)` sur un horodatage sans décalage, `Date.now() + 86_400_000`) qu'un site corrigé aurait déplacées ? Un
   site corrigé dont la valeur **envoyée** a changé autrement que par la date par défaut ?
2. **Fichier par fichier** (rollout) : chaque site corrigé prend-il `todayLocal()` (ou `addDaysIso(todayLocal(), 1)`),
   chaque affichage lit-il l'horodatage **entier** ; les `paidAt` (a, f, i) laissés ; `JournalEntryForm` ; le rappel
   manuel (`reminderSentAtDateTime`) ; la liste des utilisateurs (`'—'` d'une date illisible). Chaque test épingle-t-il
   le fuseau **et** l'horloge (`pinClock`, `pinTimeZone`) ? Un test qui passerait sans le correctif ?
3. **#888** : boucle, arrêt (`!(offset < total)`, page vide), déduplication, ordre ; relecture après un lot ; un
   `total` absent ; la limite de 500 au refus (manuel).
4. **#872** : orphelins (parent hors `projects`), marque « (archivé) » ; la création (`/supplier-invoices/new`) ne
   change pas ; un actif sous racine archivée (impossible par l'API, C-v0174c-18).
5. **#869 et #817 à l'écran** : `lockedThrough` lu dans `details` (page `supplier-invoices/[id]`) ; repli sans lui ; l'avis
   du pays inconnu (édition seulement, disparaît au changement du champ, ne bloque rien) ; `addressCountryKnown`
   **requis** dans `ContactResponse` et `npm run check` (`check-final.log`).
6. **i18n** : trois clés, quatre locales, repli = FTL fr-CH mot pour mot ; `CLES_RELEVEES` 401, `sitesTotal` 2829,
   `CANDIDATES_ATTENDUES` 77 et leurs entrées d'historique.
7. **Mutations** MC-888, MC-888-b, MC-889-a, MC-889-b, MC-889-c/g, MC-872, MC-869, MC-817 : tuées (`mut/*.log`) ?
8. **E2E et captures** : la spec `v0174-correctifs.spec.ts` (205 factures ; verrou posé **et levé** ; projets ; refus `XX` ;
   l'avis `SU` obtenu par réécriture de la réponse — choix déclaré) ; les captures montrent-elles ce que la fiche
   demande ?
9. **Manuel et PDF** contre les écrans ; le CHANGELOG `[0.17.4]`.

## Rapport

Pour chaque finding : sévérité, fichier:ligne, preuve copiée, remède. Puis la liste des axes **exercés** et **non
exercés** (raison). Un « 0 finding » sans elle ne compte pas. Décompte recompté depuis la liste.
