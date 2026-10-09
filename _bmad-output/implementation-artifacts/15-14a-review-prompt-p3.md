# Prompt — revue de code P3, Story 15-14a

*Versionné le 2026-10-09. Trois lentilles (Sonnet), contexte frais chacune, en lecture seule (bmad-code-review :
Blind Hunter, Edge Case Hunter, Acceptance Auditor). Rotation D6 : passes complètes Sonnet ↔ Opus ; Haiku en passe ciblée.*

Worktree `/home/gcorbaz/devel/kesh-15-14`, branche `story/15-14-lot-documentation-libelles`. **Le diff à revoir** :
`git -C /home/gcorbaz/devel/kesh-15-14 diff 803f3e15..0b2e5594 -- . ':(exclude)_bmad-output'` (un seul diff aplati ; manuels
`.tex` et PDF, catalogues des quatre locales, replis Rust et Svelte, gardes de texte, CHANGELOG — les fiches
`_bmad-output/` sont le contexte, pas l'objet). **Fiche** : `_bmad-output/implementation-artifacts/15-14a-manuels-et-libelles.md`
(AC, tâches, gardes G1–G13, inventaires par commande avec bloc `E`, Dev Agent Record, Change Log) ; index
`15-14-lot-documentation-libelles.md`. La fiche **15-14b**, sur la même branche, n'est PAS développée : ce qu'elle
prescrit n'est pas un manque. Issues fermées (par `gh api repos/guycorbaz/kesh/issues/N`) : #539 #547 #488 #291 #458
#449 #432 #569 #321 #323 (refs #459 ; #585 hors périmètre). Registre : `epic-15-choix-autonomes.md` (C-15-14-1 à 42).
Règles : `CLAUDE.md`.

**Troisième passe.** P1 (Sonnet ×3) 3 MEDIUM → P2 (Opus ×3) 2 MEDIUM, nés de P1 ; rapports
`/home/gcorbaz/devel/kesh-gate-logs/15-14a-review-p{1,2}-{B,E,A}.md`. La P2 est remédiée par `4a83a3dc` (code, gardes,
docs), `70c13ec9` (PDF), `21945b9a` (clippy, dernier commit de code), `0b2e5594` (Change Log) ; branche rebasée sur
`803f3e15`. **Relis d'abord `git diff 35614744..0b2e5594`** (rebase compris, le diff d'intention est celui de
`4a83a3dc`) : la famille « rouvre les exercices clôturés » bornée aux **postérieurs** (16 valeurs, replis
`errors.rs` réécrits depuis le catalogue, `blocker-messages.ts:94`, `settings/fiscal-years/+page.svelte:355`,
`user-manual.tex:684`), garde **G8-bis** (`les_prescriptions_de_reouverture_sont_bornees`, `loader.rs`), **G9** sur
les replis paramétrés, **G13** à 10 clés ; écran *Comptes bancaires* sur « 1010 Poste, 1020 Banque » (4 locales,
replis, doc-comments), **G4-bis** sur les catalogues et **G4-ter** (`la_forme_libre_nnnn_nom_est_juste_partout`,
exclusions déclarées) ; trois analyseurs de catalogue alignés avec leurs anti-tests-muets. Choix C-15-14-49 à 52.
Questions : chaque texte neuf est-il vrai au code ? Une garde neuve rougirait-elle à tort (NPA, montants, années,
un compte légitime absent des plans livrés, une clé future) ou passerait-elle à vide ? Les exclusions de G4-ter
cachent-elles un site affiché ? La grammaire des 16 valeurs neuves en de/it/en tient-elle ?

Ce qui change : textes du manuel utilisateur et d'administration qui décrivaient autre chose que le code (plans
comptables, imports, TVA, boîte de réception, multi-société, réglages, réouverture d'exercice), libellés i18n des
quatre locales et leurs replis (#569 : la réouverture prescrite dans l'ordre ; vocabulaire de clôture de-CH),
`vat_rates.rs` (`mod tests`), gardes neuves `textes_coherents.rs` (G1–G12) et le `describe` G13 de
`i18n-repli-divergent-actif.test.ts`, `regex` en dev-dependency.

Axes : chaque texte neuf est-il VRAI contre le code (relire le code qu'il décrit, pas la fiche) ? Les inventaires
(rejoue les commandes de la fiche, bloc `E`, `LC_ALL=C.UTF-8`) laissent-ils un site non résolu ? Chaque garde
rougirait-elle sur la régression qu'elle prétend empêcher, et resterait-elle verte sur un texte juste (faux rouge
sur une clé future) ? Les textes de/it/en sont-ils grammaticalement justes et cohérents avec le glossaire
(`docs/i18n-glossaire.md`) ? Les PDF régénérés correspondent-ils au `.tex` ?

## Lentilles

- **B — Blind Hunter** : le diff seul, sans la fiche. Un texte neuf qui affirme une chose fausse au code (relire le
  code décrit), une contradiction entre deux textes du diff, une garde qui passerait à vide ou rougirait à tort, un
  repli qui diverge de son catalogue, du LaTeX cassé, code mort, duplication (DRY), doc-comments devenus faux.
- **E — Edge Case Hunter** : les bornes des gardes (normalisation des apostrophes, ligatures, macros LaTeX, césures
  des PDF, continuations `\` des chaînes Rust, clé ajoutée demain dans une seule locale) ; les quatre locales clé
  par clé ; et surtout les **sites NON modifiés qui devraient l'être** — rejoue les commandes d'inventaire et cherche
  hors de leur périmètre (`website/`, `docs/`, README, frontend, replis Rust) la même affirmation fausse.
- **A — Acceptance Auditor** : chaque AC de la fiche contre le code ET les tests (un AC sans test qui le prouve est un
  finding) ; le Dev Agent Record ne déclare-t-il que ce qui a tourné (chiffres recomptés :
  `grep -c '#\[sqlx::test\]\|#\[test\]\|#\[tokio::test\]'` aux deux bornes ; 30 mutations déclarées) ; le **manuel** (`docs/manual/fr/*.tex` **et PDF aplatis** : `pdftotext -nopgbrk f.pdf - | tr '\n' ' ' | tr -s ' '`,
  ligatures ﬀ/ﬁ/ﬂ normalisées), `docs/api-external.md`, CHANGELOG, i18n 4 locales.

## Ce que tu rends

Rapport complet dans `/home/gcorbaz/devel/kesh-gate-logs/15-14a-review-p3-<B|E|A>.md` : findings numérotés, sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` pour toute affirmation de présence ou
d'absence ; code cité relu), correction proposée ; ⛔ **axes exercés ET non exercés**. Dernier message : le chemin, le
bilan par sévérité, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans
une base, ni aucune commande qui compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`),
`make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`/`apply`, `sqlx`, `cargo`, `npm`, `npx`,
`gh issue create`/`comment`/`edit`, SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`,
`gh api` en lecture, `pdftotext` vers la sortie standard.
