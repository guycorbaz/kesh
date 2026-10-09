# Prompt — revue de code P2, Story 15-14a

*Versionné le 2026-10-09. Trois lentilles (Opus), contexte frais chacune, en lecture seule (bmad-code-review :
Blind Hunter, Edge Case Hunter, Acceptance Auditor). Rotation D6 : passes complètes Sonnet ↔ Opus ; Haiku en passe ciblée.*

Worktree `/home/gcorbaz/devel/kesh-15-14`, branche `story/15-14-lot-documentation-libelles`. **Le diff à revoir** :
`git -C /home/gcorbaz/devel/kesh-15-14 diff origin/main...f1b0e0c0 -- . ':!_bmad-output'` (un seul diff aplati ; manuels
`.tex` et PDF, catalogues des quatre locales, replis Rust et Svelte, gardes de texte, CHANGELOG — les fiches
`_bmad-output/` sont le contexte, pas l'objet). **Fiche** : `_bmad-output/implementation-artifacts/15-14a-manuels-et-libelles.md`
(AC, tâches, gardes G1–G13, inventaires par commande avec bloc `E`, Dev Agent Record, Change Log) ; index
`15-14-lot-documentation-libelles.md`. La fiche **15-14b**, sur la même branche, n'est PAS développée : ce qu'elle
prescrit n'est pas un manque. Issues fermées (par `gh api repos/guycorbaz/kesh/issues/N`) : #539 #547 #488 #291 #458
#449 #432 #569 #321 #323 (refs #459 ; #585 hors périmètre). Registre : `epic-15-choix-autonomes.md` (C-15-14-1 à 42).
Règles : `CLAUDE.md`.

**Deuxième passe.** P1 (Sonnet ×3 : B 1 MEDIUM, E 2 MEDIUM, A 0 ; rapports
`/home/gcorbaz/devel/kesh-gate-logs/15-14a-review-p1-{B,E,A}.md`) remédiée par `f3e8e0e8` (dernier commit de code),
`89b4f098` (PDF), `f1b0e0c0` (Change Log) ; branche rebasée sur `f2c5e419` (15-13b). **La sévérité se déplace vers la
dernière remédiation : relis d'abord `git show f3e8e0e8`** — onze numéros de compte d'exemple corrigés au manuel
utilisateur et au guide, garde neuve **G4-bis** (`les_comptes_cites_en_exemple_existent_dans_les_plans_livres` :
tout nombre isolé de quatre chiffres doit être un compte d'un des trois plans — faux rouge sur une année, un code
postal, un montant, un numéro d'issue ?), G2 élargi à toute forme de « Réglages », §3 du guide réécrit sur les sept
étapes de l'onboarding, « exercices postérieurs » dans `error-fiscal-year-reopen-blocked` (quatre locales), analyseurs
de catalogue qui joignent les continuations. Choix C-15-14-43 à 48.
⚠️ **Résidu connu, à juger** : C-15-14-47 renvoie à la « 15-12b » la même formule trop large (« un administrateur
rouvre [d'abord] les exercices clôturés ») dans `error-fiscal-year-create-later-closed` (quatre locales + repli
`errors.rs:1539`), le repli `errors.rs:2925`, `blocker-messages.ts:94`, `settings/fiscal-years/+page.svelte:355`,
`user-manual.tex:684`. **Or la 15-12b est mergée** (`dc4bc58b`) : ce renvoi est caduc. Dis si ces sites sont faux au
code (`fiscal_years.rs`, `find_later_closed*`) et s'ils doivent être corrigés dans la 15-14a.

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

Rapport complet dans `/home/gcorbaz/devel/kesh-gate-logs/15-14a-review-p2-<B|E|A>.md` : findings numérotés, sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (sortie de `grep -nF` pour toute affirmation de présence ou
d'absence ; code cité relu), correction proposée ; ⛔ **axes exercés ET non exercés**. Dernier message : le chemin, le
bilan par sévérité, une ligne par MEDIUM+.

## Interdits

⛔ N'écris aucun fichier hors `/home/gcorbaz/devel/kesh-gate-logs/`. Aucune commande qui écrit dans le dépôt ou dans
une base, ni aucune commande qui compile ou exécute des tests : `scripts/*` (dont `scripts/prepare-release.sh`),
`make`, `latexmk`, `git commit`/`add`/`checkout`/`stash`/`apply`, `sqlx`, `cargo`, `npm`, `npx`,
`gh issue create`/`comment`/`edit`, SQL d'écriture. Autorisés : lecture, `grep`, `sed -n`, `git log`/`show`/`diff`,
`gh api` en lecture, `pdftotext` vers la sortie standard.
