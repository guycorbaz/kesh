# Revue de code P1 — Story v017-4b (facture fournisseur en brouillon, écran) — #849, #853, #830, #835, #831

*Prompt versionné (`CLAUDE.md` § *La passe ciblée* : une passe se rejoue et se conteste). Trois lentilles, **B**
(Blind Hunter), **E** (Edge Case Hunter), **A** (Acceptance Auditor), chacune en **contexte frais**, modèle **Opus**.
Langue : français soigné.*

## Le périmètre

- Worktree : `/home/gcorbaz/devel/kesh-0175`, branche `story/v017-4-facture-fournisseur-brouillon`.
- Le diff à relire : `git diff 1d9f81a5..HEAD` **hors** `_bmad-output/` (la 4a, revue et close, est en deçà de
  `1d9f81a5`). Il porte le frontend (liste, fiche, page d'édition, formulaire et ses helpers, écran de clôture, écran des
  lots), les quatre catalogues (`crates/kesh-i18n/locales/*/messages.ftl`), le glossaire (`docs/i18n-glossaire.md`), les
  specs E2E, le manuel utilisateur et son PDF, la brochure et son PDF, le CHANGELOG, le README.
- Les fiches à lire en entier : `_bmad-output/implementation-artifacts/v017-4-facture-fournisseur-brouillon.md`
  (parente : contexte, contrat de l'API, codes, précédence, AC-21 à AC-32, mutations M21 à M35, limites L1 à L8),
  `v017-4b-facture-fournisseur-brouillon-ecran.md` (sous-fiche et Dev Agent Record), `v017-4-choix-autonomes.md`
  (C-v017-4-52 à 62 pour la 4b).
- Lentille **B** : le diff seul, sans les fiches (sauf pour situer un fichier) — défauts de code Svelte et TypeScript,
  états réactifs, gestion des erreurs, textes faux, doc-comments et commentaires faux, régressions sur les écrans voisins.
- Lentille **E** : chaque branche et chaque borne du code neuf — statut (`draft`, `open`, `paid`, `cancelled`) × rôle
  (Administrateur, Comptable, Consultation) ; document (aucun, né d'une complétion avec ou sans QR, joint après coup,
  lien antérieur à la 0.17.0) ; dates (vide, égale, antérieure, refus serveur) ; montants (virgule, point, vide,
  illisible, quatre décimales, quantité vide) ; refus de chaque geste (validation, dévalidation — chaque code de la
  précédence —, suppression, `OPTIMISTIC_LOCK_CONFLICT`, `ILLEGAL_STATE_TRANSITION`) ; liste avec et sans annulées.
- Lentille **A** : chaque AC (21 à 32, la part 4b de l'AC-30), chaque mutation M21 à M35 (M21-bis compris), le Dev
  Agent Record **contre ce qui a tourné** (journaux de `/home/gcorbaz/devel/kesh-gate-logs/v0174b/` : `rouges-vitest.log`,
  `vitest-*.log`, `check-*.log`, `nextest-*.log`, `e2e-*.log`, `mut/mutations.log` et `mut/M*.out`, `make-user-1.log`,
  `user-flat.txt`), les captures (`/home/gcorbaz/devel/kesh-gate-logs/v0174b/captures/`).

## Les axes obligatoires (chaque lentille, sous son angle)

1. **Le parcours de la facture de la rc1.** Guy tient ses livres sur la `0.17.0-rc1` : une facture **validée** de la
   rc1 (statut `open`, écriture d'achat, document lié avec `linkOrigin` nul, ou sans document) doit pouvoir, **par
   l'écran**, se **dévalider**, se **corriger** (page d'édition : quels champs, quel mode de document, quel scan), puis se
   **revalider** — c'est le cas de #849 (un total erroné). Suivre ce parcours dans le code : la fiche offre-t-elle
   *Dévalider* à ce statut et à ce rôle, la page d'édition accepte-t-elle ce brouillon, les coordonnées d'une facture née
   d'un QR avant la 0.17.0 sont-elles bien modifiables (L7), le corps du `PUT` est-il celui que le serveur accepte ?
   L'écran ne réécrit-il rien d'office (aucun appel qui modifie une facture sans geste de l'utilisateur) ?
2. **Le contrat consommé.** Chaque appel de l'écran contre les routes commitées de la 4a (`crates/kesh-api/src/lib.rs`,
   `routes/supplier_invoices.rs`, `errors.rs`) : verbes, corps (`version`), réponses relues (`sourceDocument`,
   `purchaseJournalEntryId` nul), codes et `details` (`documentNumber` des trois motifs propres à la pièce,
   `invoiceDate`/`dueDate` de l'échéance), filtre `status` de la liste (valeurs, liste vide).
3. **i18n, quatre locales.** Chaque clé neuve ou modifiée aux **quatre** catalogues, le repli fr-CH égal au catalogue,
   les clés du serveur réemployées mot pour mot (`error-supplier-invoice-*`, `supplier-invoices-cancel-blocked-draft`),
   l'ordre et la borne des prescriptions de réouverture ; règle Q4 : **aucun** texte de l'écran, des catalogues, du
   manuel ou du CHANGELOG neuf ne nomme plus l'état `open` d'une facture fournisseur « ouverte » / « offen » /
   « aperta » / « open » (prose) — tri **par la valeur** (`grep -rn -i 'ouvert'`, `'offen'`, `'apert'` sur
   `crates/kesh-i18n/locales`, `frontend/src`, `frontend/tests/e2e`, `docs/`, `README.md`, `website/`) ; les
   traductions allemande, italienne et anglaise disent-elles la même chose que le français ? Les décomptes figés des
   gardes (`i18n-keys.test.ts` `sitesTotal`, `i18n-un-repli-par-cle.test.ts` `CLES_RELEVEES`,
   `i18n-libelle-en-dur.test.ts` `CANDIDATES_ATTENDUES` et la ventilation) sont-ils recomptés depuis la source, avec
   leur historique ?
4. **Le formulaire.** #853 (date vide avec un document, son message propre, l'échéance antérieure refusée sous son champ
   et au serveur, la date de l'`initial` qui prime), #830 (lecture unique par `parseAmount`, quantité vide, prix vide,
   montant attendu), #835 (total vivant et montant du QR aux décimales exactes), le bouton (« Enregistrer le
   brouillon » / « Enregistrer »), le scan **si et seulement si** `document === null && !hasDocument`, le `Pick` des douze
   champs du document. Une régression de la page de complétion (`/supplier-invoices/new?document=…`) ou de la création
   sans document ?
5. **Les specs E2E.** La spec neuve (`supplier-invoice-draft-cycle.spec.ts`, étapes par l'écran) et les specs adaptées
   (inventaire de l'AC-30, plus `fiscal-year-closing-journey.spec.ts` et `fiscal-years.spec.ts`) : prouvent-elles ce
   qu'elles disent, ou passent-elles à vide (une étape qui ne se produit pas, un sélecteur qui ne vise rien) ?

Et, comme toute passe sur du code visible (`CLAUDE.md` § *Le prompt d'une passe doit NOMMER le manuel*) : le **manuel
utilisateur** (`docs/manual/fr/user-manual.tex`) **et son PDF** (`docs/manual/fr/user-manual.pdf`), lu aplati —
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (l'apostrophe y est typographique, et la
ligature « ﬀ » remplace « ff ») — : § *Saisir une facture fournisseur* (brouillon, dates, montants), § *Le brouillon :
modifier, valider, supprimer* et *Dévalider une facture validée* (empêchements et leur levée, trou de numérotation, L4,
L7, L8, le scan à l'édition), § *Une facture fournisseur au total sous le centime*, § *Annuler une facture fournisseur*
(annuler ou dévalider, factures annulées masquées, L6), § *Numérotation* (trois chemins), § *Préparer la clôture*
(huit contrôles, K8), § *Documents à traiter*, la réconciliation d'un débit et les lots (« validée », jamais un
brouillon), le dépannage. Chaque affirmation contre le code. La **brochure** (`marketing-brochure.tex` et son PDF), le
**CHANGELOG** (`[0.17.0] — Non publié`, *Ajouté* et *Corrigé*) et le **README** (`:42`, `:43`, `:47`).

## Le rapport

- Lentille B : `/home/gcorbaz/devel/kesh-gate-logs/v0174b-review-p1-B.md` ; E : `…-E.md` ; A : `…-A.md`.
- Chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), **ligne lue citée**, défaut, remède.
  Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée.
- **Axes exercés et axes non exercés**, listés : un « 0 finding » sans cette liste ne compte pas.
- Décompte final recompté depuis la liste.

## Les interdits — lecture seule stricte

- Aucune commande qui écrit, compile ou exécute : **ni `cargo`, ni `npm`, ni `npx`, ni `make`, ni `python3`, ni
  `docker`, ni `curl`, ni `claude`**, ni aucun script du dépôt (`scripts/*`, dont `scripts/prepare-release.sh`), ni
  `sed -i`, ni `rm`, `pkill`, `killall`.
- **Aucune redirection** (`>`, `>>`, `tee`), aucun fichier temporaire, aucun fichier dans le scratchpad : un fichier
  créé par erreur se déclare au rapport, il ne se supprime pas. Le seul fichier écrit est le rapport, par l'outil
  d'écriture. `pdftotext` n'écrit que vers la sortie standard (`-`).
- Aucun `git checkout`, `switch`, `reset`, `stash`, `commit`, `push` ; `git diff`, `git show`, `git log` seulement.
- Aucune donnée réelle ; aucune base du NAS.
