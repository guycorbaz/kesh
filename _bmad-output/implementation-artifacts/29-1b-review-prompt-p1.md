# Prompt — revue de code P1, Story 29-1b (contre-passer au début de l'exercice suivant)

*Versionné le 2026-10-14. Trois lentilles **Opus** (B, E, A), contexte frais chacune, lecture seule.*

Dépôt (worktree) `/home/gcorbaz/devel/kesh-29-1`, branche `story/29-1b`.
**Le diff à revoir** : `git diff b4abae5d 64cc5835 -- . ':(exclude)_bmad-output'` — `b4abae5d` = `origin/main` (la
29-1a fusionnée) ; trois commits : `5c853450` (serveur, catalogues, tests Rust), `806598ac` (écran, Vitest, E2E) —
**dernier commit de code** —, `64cc5835` (manuel et brochure avec leurs PDF, README, `api-external.md`, CHANGELOG).
Les commits qui suivent ne portent que la fiche, le registre, `sprint-status.yaml` et ce prompt.
⚠️ Pathspec `':(exclude)…'` et **non** `':!…'`. Fiche : `_bmad-output/implementation-artifacts/29-1-preparer-cloture.md`
— **partie b seulement** : AC10 à AC18 (l'AC19 est retirée) ; tâches Tb1-Tb8 ; Dev Notes § *Concurrence* (puce 29-1b
et l'angle mort de l'exercice intercalé), § *État actuel des fichiers modifiés*, § *Tests* (T-b1 à T-b9) et
§ *Mutations* (M-b1 à M-b12) ; et le **Dev Agent Record**, section *29-1b* (tests rouges, gates, mutations, décomptes,
captures, écarts). Choix C-29-1-48 à C-29-1-58 au registre `epic-29-choix-autonomes-29-1.md` (nés du développement ;
C-29-1-1 à 31 sont ceux de la fiche, 32 à 47 ceux de la 29-1a). Règles : `CLAUDE.md` du dépôt — § « Recompter ses
propres comptes rendus », § « Propagation post-patch », § « Le prompt d'une passe doit NOMMER le manuel », § « Une
passe qui ne déclare pas ses axes exercés ne compte pas », § *Pattern batch* (sans objet ici : à confirmer),
§ *Migration breaking policy* (aucune migration attendue : à vérifier). Issue : #232 — **la PR de la 29-1b la
fermera** (`closes #232` au titre et au corps de la PR seulement) ; aucun `closes` / `fixes` dans un **message de
commit**, même en prose : à greper, `git log b4abae5d..HEAD --format=%B | grep -niE "close[sd]?|fix(e[sd])?"`.

⛔ **DONNÉES PERSONNELLES — DÉPÔT PUBLIC.** N'ouvre aucun fichier hors du dépôt et de
`/home/gcorbaz/devel/kesh-gate-logs/`. Les captures `kesh-gate-logs/29-1b-captures/*.png` sont synthétiques (IBAN de
test, montants ronds) : tu peux les ouvrir.

## Axes communs aux trois lentilles (chacune les déclare exercés ou non)

1. **La datation, paramètre interne de la contre-passation unique (AC10, AC11).**
   `crates/kesh-db/src/repositories/journal_entries.rs` : `ReversalDating`, `reverse_in_tx_inner` et ses quatre
   entrées (`reverse_in_tx`, `reverse_owned_in_tx` en `Today` ; `reverse_at_next_fiscal_year_start_in_tx` et
   l'enveloppe pool `reverse_at_next_fiscal_year_start`, sans autorité). Aucune copie de la fonction. Étape (1) : la
   fin d'exercice lue par la **même** requête verrouillante. Étape (4) : `find_next_after` (lecture ordinaire) **puis**
   `find_open_covering_date` (verrouillante) ; ⛔ le `status` de la lecture ordinaire n'est **jamais** lu (C-29-1-27) ;
   la garde `fy.id != next.id` (C-29-1-50) est-elle un jugement de statut déguisé ? Les refus `NextFiscalYearMissing`
   (exercice **de l'origine** nommé) et `NextFiscalYearClosed` (exercice suivant nommé), leur mapping `409` dans
   `crates/kesh-api/src/errors.rs`, leurs `details`. La suite est-elle **celle du jour** (création, refus
   `PeriodLocked` / `LaterFiscalYearClosed`, lettrage de l'étape (5), relecture (6)) ? L'audit : action
   **littérale**, chemin du jour **octet pour octet** (`{"reversalJournalEntryId": …}`), chemin neuf + `"dating"`.
2. ⛔ **B5 — verrous et naissance de la vue (AC12) — axe OBLIGATOIRE pour la lentille E.** Le chemin
   `NextFiscalYearStart` prend-il **les mêmes verrous, dans le même ordre**, que `Today` — origine et son exercice
   (1), lignes (2), parcours de `find_open_covering_date` (4), puis ceux de `create_in_tx_inner` ? La **seule**
   lecture ajoutée est-elle ordinaire et placée **après** l'appel `reversal_blockers(` ? Sur le chemin sans autorité,
   la vue naît-elle **exactement** au recensement (étape (1) verrouillante, étape (1-bis) non exécutée) — relis
   chaque lecture entre l'étape (1) et le recensement, `grep -nF` à l'appui. Avec `find_next_after` placé après
   `reversal_blockers`, une lecture ordinaire **antérieure** existe-t-elle ailleurs sur le chemin (dans
   `reversal_blockers`, `document_owners`, `archived_accounts_in_tx`) qui changerait la conclusion ? Le doc-comment de
   `reverse_in_tx_inner` dit-il vrai : la phrase sur la sûreté (confirmation verrouillante, second refus
   `FISCAL_YEAR_CLOSED`), « conséquence utile, non une garantie » (attente de `close(N+1)` **si N est le premier
   exercice**), le cycle avec `close` quand un M antérieur existe, la réouverture de N+1, le cycle avec `create` d'un
   exercice **postérieur** (P2 F L4), l'angle mort de la confirmation non verrouillante, l'angle mort de l'exercice
   intercalé (chemin pool / chemin `_in_tx`) ? Les cycles nommés sont-ils réels et résolus par le rejeu des **deux**
   côtés (route neuve `Rejouee`, `close` et `create` rejouées) ? Cherche un cycle **neuf** que la datation créerait.
3. **La route et ses compteurs (AC13).** `POST /api/v1/journal-entries/{id}/reverse-at-next-fiscal-year-start` dans
   `comptable_routes` (`crates/kesh-api/src/lib.rs`), sans corps, handler rejoué (`retry_on_deadlock`, nom
   d'opération), `201 JournalEntryResponse` ; Consultation `403`, clé `read-write` admise, `404` autre société.
   Registre `crates/kesh-api/tests/audit_route_registry.rs` : entrée `Traced, Rejouee`, compteurs 118 → 119, 112 → 113,
   121 → 122 (deux sites), 24 → 25 (**deux** sites, dont `examinees` l. ~876 que la fiche ne nommait pas,
   C-29-1-55), messages et doc du fichier ; greper la **valeur** (`\b(118|112|121)\b` sur le dépôt, `\b24\b` sur ce
   fichier et `docs/api-external.md`).
4. **L'écran (AC15, AC16).** `frontend/src/routes/(app)/journal-entries/[id]/+page.svelte`,
   `frontend/src/lib/features/journal-entries/next-year-reversal.ts`, `journal-entries.api.ts` : la liste des
   exercices lue à l'ouverture (jeton, `Promise.resolve`, réponse non-tableau), le bouton sous les conditions de
   *Contre-passer* **et** un suivant ouvert, la table des cinq états de l'AC15 (attente : ni bouton ni motif ; échec ;
   aucun suivant ; suivant clos nommé ; ouvert) et le sixième état de C-29-1-48 ; Consultation : ni bouton ni motif ;
   `reversable` faux : ni l'un ni l'autre ; *Contre-passer* présent quand la liste échoue ; `startEdit` garde sa
   propre lecture. Le dialogue **unique** (C-29-1-49) : le geste du jour est-il **inchangé** (texte, `data-testid`,
   comportement d'erreur) ; le mode « exercice suivant » peut-il s'afficher avec un exercice qui n'est plus ouvert, ou
   appeler une route avec le texte de l'autre ? La confirmation nomme-t-elle la date au format suisse et l'exercice ;
   un refus s'affiche-t-il traduit, jamais « [object Object] » (#520) ? i18n : sept clés
   `journal-entries-reverse-next-year-*` et deux `error-next-fiscal-year-*` × 4 locales, traduites réellement, replis
   égaux à fr-CH, G9 étendue des deux clés (`textes_coherents.rs`), `lint-i18n-ownership`, borne `i18n-keys.test.ts`
   (2163 → 2170).
5. **Les tests prouvent-ils ce qu'ils nomment.** T-b1 à T-b9 contre la fiche :
   `crates/kesh-db/tests/reversal_next_fiscal_year.rs` (T-b1, T-b3, T-b5 (i) et (ii), T-b6), la garde T-b4 (module de
   tests de `journal_entries.rs` — que voit-elle exactement, et que ne voit-elle pas ?), T-b2 et T-b7
   (`journal_entry_reversal_e2e.rs`), T-b8 (`journal-entry-page.test.ts`), T-b9 (E2E
   `frontend/tests/e2e/fiscal-year-closing-journey.spec.ts`). T-b5 : la vraie fonction est-elle exercée, jamais une
   reconstitution ; les motifs `attendre_une_requete_en_cours` distinguent-ils ce qu'ils prétendent (« B attend N »
   vs « B attend N+1 » ; « A attend à l'étape (1) ») ; le texte recopié de `LOCK_EARLIER_BY_ID_SQL` /
   `LOCK_IN_COMPANY_SQL` est-il identique à la source ? La campagne M-b1 à M-b12 (journal
   `kesh-gate-logs/291b-mutations.log`, script cité au Dev Agent Record) — cherche la **mutation plausible qui
   resterait verte** (exemples : `find_next_after` remplacé par `find_open_covering_date(end_date + 1)` ; la
   confirmation faite sur `origin_fy_end` ; le `dating` d'audit sur un mauvais chemin ; le motif « clôturé » affiché
   sans le nom ; `nextYearReversal` qui prendrait le **dernier** suivant ; le jeton de liste retiré).
6. ⛔ **LA DOCUMENTATION — MANUEL UTILISATEUR ET BROCHURE (PDF compris), README, `docs/api-external.md`, CHANGELOG.**
   Chaque phrase neuve ou modifiée dit-elle vrai **au code** ? Manuel utilisateur (`docs/manual/fr/user-manual.tex`) :
   la sous-section *Les transitoires de fin d'exercice* (`\label{sec:transitoires}`) — l'usage (1300 « Actifs
   transitoires », 2300 « Passifs transitoires », garde G4-bis), la marche, ce que Kesh exige (créé et ouvert, premier
   jour hors période verrouillée, aucun exercice clôturé après), « ce qui se fige » (une écriture contre-passée **et**
   sa contre-passation ne se modifient ni ne se suppriment : vrai au code ?), l'angle mort de la transitoire oubliée ;
   l'exception au verrou de période (`keshwarning` du § *Verrou de période*) ; la phrase du § *Exercices dans le
   désordre* et le paragraphe « Des boutons offerts, puis refusés au clic » ; le renvoi depuis *Contre-passer* et
   depuis *Préparer la clôture* (K7, et la limite neuve). Cherche une phrase du manuel, de la brochure, du README ou de
   `website/` que la story rend **fausse** et n'a pas touchée (`grep -rn "datée du jour\|daté du jour\|celle du jour"
   docs/ README.md website/ crates/` — chaque résultat restant doit parler du geste du jour ou d'une annulation).
   Contrôle les **PDF aplatis** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (et
   `marketing-brochure.pdf`) vers `/home/gcorbaz/devel/kesh-gate-logs/29-1b-review-p1-<lettre>-*-flat.txt` —
   ⚠️ `pdftotext` rend les ligatures, l'apostrophe typographique « ’ » et coupe les mots en fin de ligne
   (« contrepassation ») : un `grep` naïf rend un faux négatif. Les PDF sont-ils ceux du HEAD ? Brochure
   (`marketing-brochure.tex` l. ~418) : plus de « à venir », aucune écriture de clôture annoncée (écartées par
   conception, C-29-1-2). README (l. 47) : contrôles préalables et transitoires, lien `#232` (garde G7). `api-external.md` :
   la sous-section au § 7 (sans corps, clés `read-write`, date, `201`, refus et leurs `details`, statuts `400` de
   `PERIOD_LOCKED` et `LATER_FISCAL_YEAR_CLOSED`), les deux lignes de la table d'erreurs, la liste des routes
   rejouées. CHANGELOG : **une seule** section `## [0.14.0] — Non publié`, au-dessus de `## [0.13.4] — Non publié`
   (forme attendue par `scripts/prepare-release.sh`, **sans l'exécuter**), entrée de #232.
7. **Gardes et décomptes.** `lint-i18n-ownership`, `i18n-keys.test.ts`, G4-bis, G7, G9 (`textes_coherents.rs`) ;
   **recompte depuis la source** tout décompte du Dev Agent Record (tests neufs par fichier aux **deux** bornes
   `b4abae5d` / dernier commit de code, clés i18n, fichiers au `git diff --stat`, mutations tuées) et vérifie le
   **périmètre** déclaré de chaque nombre ; les gates déclarés contre les journaux `kesh-gate-logs/291b-*`.

## Lentilles

- **B — Blind Hunter : bugs et états**, sans présupposer que la fiche a raison. Une origine dont l'exercice a été
  **supprimé** (`onboarding::reset`) entre deux lectures ; un exercice suivant **gappé** ; deux exercices suivants
  dont l'un clos ; une origine qui est elle-même dans N+1 (le suivant est N+2) ; une écriture d'ouverture ; le
  `fy.id != next.id` ; `entry_date` hors de N+1 ; la page : navigation d'une écriture à l'autre pendant la lecture de
  la liste, double clic sur *Confirmer*, refus puis relecture qui laisse le dialogue ouvert, `reverseConfirm` resté à
  `'nextYear'` après la relecture ; le mock d'`i18nMsg` modifié (C-29-1-56) masque-t-il un défaut d'interpolation ?
- **E — Edge Case Hunter : bords, verrous (axe 2, obligatoire), et les tests prouvent-ils ce qu'ils nomment.**
  Rejoue mentalement les deux ordres de T-b5 et cherche l'entrelacement non couvert (réouverture de N+1 pendant la
  contre-passation ; création d'un exercice intercalé ; `close(N+1)` quand un M antérieur existe) ; rejoue la
  campagne et cherche la mutation verte (axe 5) ; un test qui passerait à vide (motif `attendre…` jamais vu mais
  `interrompre` vrai ; `queryByTestId` null par sélecteur faux ; l'E2E qui ne vérifie pas que le relevé a bien produit
  une transaction) ; l'E2E (`with-company-no-fy`, années fixes 2025-2026, `fys`, nettoyage, captures).
- **A — Acceptance Auditor : conformité, décomptes, documentation.** Chaque AC de la partie b (AC10 à AC18) contre le
  code : satisfait, partiel, absent ; T-b1 à T-b9 et M-b1 à M-b12 présents et conformes à la fiche ; les écarts
  déclarés au Dev Agent Record sont-ils **tous** déclarés ; les captures (bouton, confirmation, refus ; clair et
  sombre — le « sombre » est identique, C-28-1-7) ; les chiffres de gates et d'E2E déclarés contre les journaux
  `kesh-gate-logs/291b-*` (dernier gate et E2E complet au dernier commit de code) ; les choix C-29-1-48 à 58 :
  justifiés, ou défaut ? `sprint-status.yaml` : clé `29-1b-contre-passation-transitoires` en `review`, commentaire
  daté ; CHANGELOG en v0.14.0.

## Ce que tu rends

Ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/29-1b-review-p1-<B|E|A>.md` (ta lettre). Findings avec sévérité
(CRITICAL/HIGH/MEDIUM/LOW), `fichier:ligne`, **preuve** (commande et sortie, ou code relu), correction proposée. Pour
tout finding affirmant qu'un code ou un texte est absent ou présent : la sortie d'un `grep -nF` copiée.
⛔ **La liste des axes exercés ET non exercés** (les sept axes communs et ceux de ta lentille) — un « 0 finding » sans
elle ne compte pas.

## Interdits

⛔ N'écris aucun fichier hors de ton rapport dans `/home/gcorbaz/devel/kesh-gate-logs/` (et les textes aplatis des PDF
au même endroit) ; aucune commande qui écrit dans le dépôt ou dans une base, ni qui compile ou exécute :
`scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/mem-guard.sh`), `make`, `latexmk`,
`xelatex`, `git commit`/`add`/`checkout`/`switch`/`stash`/`reset`/`rebase`/`restore`/`push`/`tag` — **aucun `git
checkout` ni `git switch` sous aucune forme**, pas même `--detach` —, `sqlx`, `cargo` (aucune sous-commande), `npm`,
`npx`, `node`, `deno`, `bun`, `python`/`python3` (pas même le script de mutations), `perl`, `ruby`, `php`, `sed -i`,
`awk` en écriture, tout autre interpréteur, `mariadb`, `mysql`, `docker`, `curl` et `wget`, `gh` en écriture,
`pkill`/`kill`, aucune requête SQL. Aucun fichier hors du dépôt. Autorisés : lecture, `grep`, `sed -n`, `wc`, `sort`,
`cmp`, `git log`/`show`/`diff`, `gh issue view`, `pdftotext` vers `/home/gcorbaz/devel/kesh-gate-logs/`, l'ouverture
des PNG de `kesh-gate-logs/29-1b-captures/`.
