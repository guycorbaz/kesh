# Validation P3 (ciblée) de la fiche 49-1c-types-de-lignes-ecran — prompt de la passe

*Passe **ciblée**, une seule lentille, contexte frais, **lecture seule**, sur **Sonnet** (P1 Sonnet, P2 Opus ; CLAUDE.md
§ *La passe ciblée*). Elle porte sur **le seul commit de la remédiation de la P2**.*

Tu valides une remédiation de la fiche de la story **49-1c-types-de-lignes-ecran** de Kesh (release 0.20.0, Epic 49,
#1242). Français soigné. Tu ne corriges rien : tu rends des findings.

- Worktree : `/home/gcorbaz/devel/kesh-49-1c`, branche `story/49-1c-types-de-lignes-ecran`.
- **Le commit visé** : celui dont le message est `docs(49-1c) : validation P2 remédiée (refs #1242)` —
  `git log --format='%h %s' -5` le nomme ; lis-le par `git show <hash>`. Ne relis pas toute la fiche : seulement ce que ce
  commit change, et les lignes de la fiche ou du code qu'il cite.
- Code de référence : **`5156f2b7b`** (`git show 5156f2b7b:<chemin> | sed -n '<l>p'`) ; `git diff --stat 5156f2b7b..HEAD --
  crates frontend docs scripts` doit être vide.
- À lire d'abord : le rapport de la P2, `/home/gcorbaz/devel/kesh-gate-logs/e491c/validation-49-1c-p2.md` (V2-1 à V2-8).
- Rapport à écrire : `/home/gcorbaz/devel/kesh-gate-logs/e491c/validation-49-1c-p3.md`.

## La lentille : « la remédiation est-elle juste, et n'a-t-elle rien cassé ? »

Pour chaque V2-n, et pour chaque hunk du commit :

1. **Levé ?** Le finding est-il réellement traité, tel que la P2 le demandait ?
2. **Prémisses relues** : chaque citation neuve se relit à `5156f2b7b` — `InvoiceForm.test.ts:1607`, `:1626`, `:1635-1649` ;
   `invoice-helpers.ts:40`, `:46` ; `invoice-helpers.test.ts:25-29` ; `backup.rs:831` ; `invoice_pdf_service.rs:88-89` ;
   `epic-49-types-de-lignes-facture.md:128-130`, `:154` ; `sprint-status.yaml:1191`, `:1195`.
3. **Défaut neuf** :
   - la signature de `computeInvoiceTotal` (D6) **compile-t-elle**, à la lecture, avec `strict`, la fixture
     `invoice-helpers.test.ts:25-29` et l'appelant `InvoiceForm.svelte:529-533` tel que D5 le réécrit ?
   - le **rejeu** d'AC8 (c) (1) est-il exécutable tel qu'écrit (copie temporaire à côté de l'original, imports relatifs,
     configuration de `vitest`, suppression de la copie) et suffit-il ? La seule exception (`AC9 a`) est-elle la seule
     nécessaire (cherche d'autres `toEqual` sur une charge de lignes dans les quatre fichiers à `5156f2b7b`) ?
   - les contrôles positifs d'AC10 (d) rougissent-ils sur un texte faux et restent-ils verts sur un texte juste ?
   - le **repli à la coupure** (Dev Notes) est-il cohérent avec le fichier d'épique (dépendances de la 49-2 et de la 49-3,
     ordre de sortie) et avec la fermeture de #1401 ?
   - la consigne pour la 49-4 (fichiers communs) est-elle exacte sur les phrases qu'elle aura à reprendre ?
4. **Propagation** : les valeurs corrigées (`backup.rs:2526`, `81-87`, la signature en `Pick<…'lineType'…>`, « qui ont fondé »,
   « sort avec elle », le motif `comment.*(réservées`) ne subsistent-elles dans la fiche **que** dans l'entrée P1 du Change
   Log, annotée par l'entrée P2 ?
5. **Manuel** : la consigne pour la 49-4 et AC10 restent-ils vrais contre `docs/manual/fr/user-manual.tex` (`:1456`,
   `:1594`) et le PDF aplati (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`) ?

## Interdits

Lecture seule. **Aucune écriture** hors de ton rapport. Aucun `cargo`, `npm`, `npx`, `vitest`, `make`, `docker`, `mariadb`,
`sqlx` ; aucun script de `scripts/` (ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni `scripts/gate-ci.sh`, ni
`scripts/recette-montee.sh`, ni aucun script mutant) ; aucun `git commit`, `git checkout`, `git switch`, `git reset`,
`git stash`, `git push`, `git merge`, `git worktree` ; aucun `gh` qui écrit. Rien sous `~/kesh/`. Aucune donnée réelle de
Guy. Ne touche à aucun autre worktree.

## Rapport

En tête : **verdict chiffré** `CRITICAL / HIGH / MEDIUM / LOW`. Puis chaque finding (`V3-n`) : sévérité, point de la
lentille, constat **avec la ligne lue citée** (`chemin:ligne` et la ligne copiée, ou la commande et sa sortie),
conséquence, remède. Puis, pour chaque V2-n : levé ou non. Puis la liste des **points exercés et non exercés** (avec la
raison). Un « 0 finding » sans cette liste ne compte pas comme passe. Pour tout finding CRITICAL ou HIGH qui affirme
l'absence d'un code, cite le `grep -nF` qui l'établit.
