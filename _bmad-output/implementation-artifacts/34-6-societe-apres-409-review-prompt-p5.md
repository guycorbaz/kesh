# Revue de code P5 — Story 34-6 (#953) : après un 409, l'édition de la société montre l'écriture concurrente

Passe **complète**, trois lentilles **Opus** en contexte frais, chacune confiée à un agent distinct. Worktree
`/home/gcorbaz/devel/kesh-e346`, branche `story/34-6-societe-apres-409`, base `665de151`. Périmètre :
`git diff 665de151..HEAD`, aplati. **Regard appuyé sur la remédiation de la P4** : `git diff 721125d3..HEAD` (rouges
`e486b722`, correctif `2c8c2241`, puis fiche et registre).

## Ce que chaque lentille lit d'abord

- La fiche `_bmad-output/implementation-artifacts/34-6-societe-apres-409.md` : Conception, AC-1 à AC-8 (dont l'AC-6 bis
  annotée), Dev Agent Record et Change Log, dont l'entrée « revue de code P4 » qui donne le sort de chaque finding.
- Le registre `e34-choix-autonomes-e346.md`, C-e346-1 à 11, en particulier C-e346-11 : sa limite est écrite dans les
  deux sens, et la règle d'adoption est maintenant une fonction pure.
- Les rapports P4 : `/home/gcorbaz/devel/kesh-gate-logs/e346-review-p4-{B,E,A}.md`.
- Le `CLAUDE.md` du worktree, § *Review Iteration Rule*.

## La remédiation à relire en priorité

- `frontend/src/lib/features/settings/company-conflict.ts` :
  - `adoptionDecision`, fonction pure : version inférieure, égale ou supérieure, et séquence retenue par `Math.max` ;
  - `PENDING_FIELD_CLASS` : `read-only:bg-surface-alt read-only:text-text-muted read-only:cursor-progress`.
- `frontend/src/routes/(app)/settings/+page.svelte` : `adoptCompany` délègue à `adoptionDecision`, l'en-tête de
  `reloadAfterConflict` est corrigé, et l'indentation des `class={PENDING_FIELD}` aussi.
- Les tests neufs :
  - dans `company-conflict.test.ts` : `adoptionDecision` (quatre cas), et le contrôle qui lit `src/app.css` pour
    vérifier que chaque couleur nommée est un jeton défini ;
  - dans `settings-page.test.ts` : « coordonnées » et « adresse : l'avis est effacé au départ du nouvel envoi ».
- Les mutations M23 à M30, dans `/home/gcorbaz/devel/kesh-gate-logs/e346/mut/`.

Questions à se poser :
- **Le contrôle du thème** : la règle qu'il applique (les préfixes `bg-`, `text-` et `border-` ; le nom du jeton capturé
  par une expression non gourmande ancrée à la fin) laisse-t-elle passer un jeton absent ? Que fait-il d'une couleur
  Tailwind par défaut, comme `slate-500` ?
- **Tailwind 4 voit-il la classe**, maintenant qu'elle vit dans un module `.ts` et non dans un `.svelte` ? Le CSS
  construit (`frontend/build/_app/immutable/assets/*.css`) contient-il `read-only\:bg-surface-alt` et
  `read-only\:text-text-muted` ?
- **`adoptionDecision`** est-elle équivalente à l'ancienne règle, à `Math.max` près ?
- **La séquence qui ne recule plus** : peut-elle refuser à tort une réponse légitime ?

## Captures

Elles sont dans `/home/gcorbaz/devel/kesh-gate-logs/e346/captures/`, en thème clair. Les **six** sont à ouvrir avec
l'outil Read sur l'image :
- `e346-adresse-conflit-et-fusion.png` ;
- `e346-coordonnees-un-champ-en-conflit.png` ;
- `e346-email-vide-ailleurs.png` ;
- `e346-coordonnees-autre-bloc.png` ;
- `e346-deux-blocs-ouverts.png` ;
- **`e346-adresse-envoi-en-vol.png`**, neuve : l'état en vol. Le fond et le texte des champs en lecture seule se
  distinguent-ils ?

## Journaux à confronter

- `suite5.out` ;
- `p4-front-check.log`, `p4-front-lint.log`, `p4-front-test-unit.log`, `p4-front-build.log` ;
- `e2e-r7.log` ;
- `mut/M23..M30.log`.

## Axes déclarés non exercés jusqu'ici, à exercer ici

- **Le rendu à largeur de téléphone** : aucune capture n'en existe. Le juger au gabarit, et le déclarer comme tel.
- **Un lecteur d'écran réel** : à juger au code (#997 et l'issue demandée en P4 pour les valeurs hors de la région vive).
- **Les journaux de mutation** M12 à M19.
- **`company-conflict.test.ts`**, cas par cas.
- **Le PDF du manuel**, aplati (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, puis `awk`) :
  reste-t-il vrai ?
- **La confrontation nominative** des sept échecs E2E à `docs/testing.md` § « Les échecs attendus ».
- **`docs/manual/fr/admin-manual.tex`** : la story le touche-t-elle, et le devrait-elle ?

## Lentilles

- **B — Blind Hunter** : le diff seul, sans la fiche. Rapport : `/home/gcorbaz/devel/kesh-gate-logs/e346-review-p5-B.md`.
- **E — Edge Case Hunter** : chaque branche, chaque ordre de réponses, et `adoptionDecision` aux bornes. Rapport :
  `/home/gcorbaz/devel/kesh-gate-logs/e346-review-p5-E.md`.
- **A — Acceptance Auditor** : AC par AC ; le sort de chaque finding P4 (traité avec test et mutation, ou issue avec son
  numéro) ; les décomptes recomptés depuis la source, avec leur périmètre. Rapport :
  `/home/gcorbaz/devel/kesh-gate-logs/e346-review-p5-A.md`.

## Règles communes

- **Lecture seule.** Aucune commande qui écrit, compile ou exécute : ni `npm`, ni `npx`, ni `cargo`, ni `make`, ni
  `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `git checkout|switch|reset|commit|stash`. Pas de
  redirection, pas de fichier temporaire : le seul fichier écrit est le rapport, par l'outil Write.
- Toute affirmation de présence ou d'absence s'appuie sur une sortie `git grep -nF` ou `/usr/bin/grep -nF` copiée.
- Sévérités CRITICAL, HIGH, MEDIUM, LOW, chacune avec fichier:ligne et preuve.
- **Le rapport déclare les axes exercés ET non exercés.** Un « 0 finding » sans cette liste ne compte pas.
- Aucune donnée réelle dans le rapport.
