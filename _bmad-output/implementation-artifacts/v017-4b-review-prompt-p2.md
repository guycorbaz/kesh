# Revue de code P2 — Story v017-4b (facture fournisseur en brouillon, écran) — #849, #853, #830, #835, #831

*Prompt versionné (`CLAUDE.md` § *La passe ciblée* : une passe se rejoue et se conteste). Passe **complète**, trois
lentilles, **B** (Blind Hunter), **E** (Edge Case Hunter), **A** (Acceptance Auditor), chacune en **contexte frais**,
modèle **Sonnet** (rotation après la P1 en Opus). Langue : français soigné.*

## Ce qui précède

La revue P1 (Opus) a rendu 5 MEDIUM et 23 LOW : `/home/gcorbaz/devel/kesh-gate-logs/v0174b-review-p1-{B,E,A}.md`.
Sa remédiation tient en trois commits : `165d9559` (WIP interrompu le 2026-10-31, jamais vérifié à sa date),
`79c2211e` et `eed15c18` (2026-11-03). La table « finding → sort » est au Dev Agent Record de la sous-fiche
(§ *Remédiation de la revue de code P1*). **Ne la croyez pas sur parole : vérifiez chaque ligne au code.**

## Le périmètre

- Worktree : `/home/gcorbaz/devel/kesh-0175`, branche `story/v017-4-facture-fournisseur-brouillon`.
- **Premier diff, à relire en premier et ligne à ligne** : la remédiation P1, `git diff 9a229bd3..HEAD` **hors**
  `_bmad-output/` (frontend, quatre catalogues, manuel utilisateur et son PDF, CHANGELOG, spec E2E du cycle, bancs).
- **Second diff**, pour le reste des axes : toute la 4b, `git diff 1d9f81a5..HEAD` hors `_bmad-output/`.
- Fiches à lire en entier : `_bmad-output/implementation-artifacts/v017-4-facture-fournisseur-brouillon.md` (parente :
  contrat, codes, précédence, AC-21 à AC-32, M21 à M35, limites L1 à L8), `v017-4b-facture-fournisseur-brouillon-ecran.md`
  (sous-fiche, Dev Agent Record, table de la remédiation P1), `v017-4-choix-autonomes.md` (C-v017-4-52 à la fin).
- Lentille **B** : les diffs seuls, sans les fiches (sauf pour situer un fichier) — Svelte et TypeScript, états
  réactifs, erreurs, textes faux, commentaires et doc-comments faux, régressions des écrans voisins (lots, clôture,
  réconciliation).
- Lentille **E** : chaque branche et chaque borne — statut (`draft`, `open`, `paid`, `cancelled`) × rôle
  (Administrateur, Comptable, Consultation) ; `lastConfirmedBatch` (nul, présent, `confirmedAt` nul) ; refus de chaque
  geste suivi d'une relecture vers chaque autre statut, et le geste suivant ; deux bascules de la case des annulées ;
  K8 avec 0, 20, plus de 20 brouillons ; document, dates, montants comme en P1.
- Lentille **A** : chaque finding de la P1 contre la table « finding → sort » et le code ; chaque AC (21 à 32, la part
  4b de l'AC-30) ; M21 à M35 et les mutations R1 à R11 de la remédiation
  (`/home/gcorbaz/devel/kesh-gate-logs/v0174b/mut-r1/mutations.log`, `mutations-2.log`, sorties `R*.out`) ; le Dev
  Agent Record **contre ce qui a tourné** (journaux `r-*.log` de `/home/gcorbaz/devel/kesh-gate-logs/v0174b/`) ; les
  captures de la remédiation (`/home/gcorbaz/devel/kesh-gate-logs/v0174b/captures-r1/`), **ouvertes une à une** : chacune
  montre-t-elle l'état qu'elle nomme ?

## Les axes obligatoires (chaque lentille, sous son angle)

1. **La remédiation P1, d'abord.** Pour chaque finding de la P1 (B-1 à B-9, E-1 à E-10, A-1 à A-9) : le correctif
   existe-t-il, fait-il ce que dit la table, est-il exercé par un test qui rougirait sans lui ? Les correctifs ont-ils
   introduit un défaut (la sévérité se déplace vers ce qu'on vient d'écrire) ? En particulier : les refus rendus **hors**
   des blocs de statut (`validateError`, `deleteError`, `unvalidateError`) — s'affichent-ils au bon endroit, et ne
   restent-ils jamais périmés ; `isStaleState` élargi — une relecture inutile ou une boucle est-elle possible ;
   l'avertissement du lot confirmé — vrai pour toute facture `open` qui porte `lastConfirmedBatch` (une facture
   dévalidée puis revalidée ? une date de confirmation nulle ?) ; le jeton de lecture de la liste ; `readableAmount` ;
   les taux expirés proposés (`extraRates`) ; les projets archivés filtrés ; le message de date sans document ; le
   libellé du refus des lots aux quatre locales ; « … et N autre(s) » de K8 ; l'exercice postérieur nommé.
2. **Le parcours de la facture de la rc1.** Une facture **validée** de la `0.17.0-rc1` (statut `open`, écriture
   d'achat, document lié avec `linkOrigin` nul, ou sans document), et le cas d'une facture payée par un lot confirmé
   dont le règlement a été annulé : par l'écran, se dévalide-t-elle, se corrige-t-elle, se revalide-t-elle ?
   L'avertissement du lot arrive-t-il **avant** que le montant change ? L'écran n'écrit-il rien sans geste ?
3. **i18n, quatre locales.** Chaque clé neuve ou modifiée par la remédiation (`supplier-invoices-unvalidate-batch-warning`,
   `-unvalidate-blocked-later-fiscal-year-closed-named`, `-err-invoice-date-plain`, `fiscal-years-checks-k8-more`,
   `payment-batches-failed-supplier-invoice-not-open`, les textes de-CH et en-CH de B-6, it-CH de A-8) aux quatre
   catalogues, repli fr-CH égal au catalogue, même sens dans les quatre langues, une seule forme d'adresse en italien,
   prescriptions de réouverture ordonnées et bornées ; règle Q4 (aucun « ouverte », « offen », « aperta », « open » en
   prose pour l'état `open` d'une facture fournisseur) **par la valeur** ; décomptes figés recomptés à la source
   (`sitesTotal` 2790 → 2794, `CLES_RELEVEES` 384 → 387) avec leur historique.
4. **Le formulaire et la liste**, comme en P1 (#853, #830, #835, #831, scan ssi `document === null && !hasDocument`),
   plus les correctifs B-7, B-8, B-9 = E-5, E-6, E-7, B-4 = E-8.
5. **Les specs E2E.** `supplier-invoice-draft-cycle.spec.ts` : la facture annulée du décor, absente puis présente
   selon la case (B-5 = E-9) ; le lot confirmé puis désengagé, nommé sur la fiche et dans la confirmation (E-2) ; K8
   amené à la vue avant sa capture (A-2). Prouvent-elles ce qu'elles disent, ou passent-elles à vide ?

Et, comme toute passe sur du code visible (`CLAUDE.md` § *Le prompt d'une passe doit NOMMER le manuel*) : le **manuel
utilisateur** (`docs/manual/fr/user-manual.tex`) **et son PDF** (`docs/manual/fr/user-manual.pdf`), lu aplati —
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` (apostrophe typographique, ligature « ﬀ ») —,
d'abord les phrases réécrites par la remédiation (§ *Le brouillon : modifier, valider, supprimer* : l'état dit par la
liste et par la fiche ; § *Dévalider une facture validée* : la facture payée, la liste des refus, l'exercice postérieur
nommé, le lot confirmé nommé sur la fiche et dans la confirmation), puis les autres § listés au prompt de la P1
(`v017-4b-review-prompt-p1.md`). Chaque affirmation contre le code. Le **CHANGELOG** (`[0.17.0] — Non publié`, la
puce du cycle, réécrite), la **brochure** et le **README** (`:42`, `:43`, `:47`).

## Le rapport

- Lentille B : `/home/gcorbaz/devel/kesh-gate-logs/v0174b-review-p2-B.md` ; E : `…-p2-E.md` ; A : `…-p2-A.md`.
- Chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), **ligne lue citée**, défaut, remède ;
  dire s'il vient de la remédiation P1 ou de la conception d'origine. Toute affirmation de présence ou d'absence
  s'appuie sur une sortie `grep -nF` copiée.
- **Axes exercés et axes non exercés**, listés : un « 0 finding » sans cette liste ne compte pas.
- Décompte final recompté depuis la liste.

## Les interdits — lecture seule stricte

- Aucune commande qui écrit, compile ou exécute : **ni `cargo`, ni `npm`, ni `npx`, ni `make`, ni `python3`, ni
  `docker`, ni `curl`, ni `claude`**, ni aucun script du dépôt ou des journaux (`scripts/*`, dont
  `scripts/prepare-release.sh` ; `kesh-gate-logs/v0174b/*.sh`, `mut*/campagne*.sh`), ni `sed -i`, ni `rm`, `pkill`,
  `killall`.
- **Aucune redirection** (`>`, `>>`, `tee`), aucun fichier temporaire, aucun fichier dans le scratchpad : un fichier
  créé par erreur se déclare au rapport, il ne se supprime pas. Le seul fichier écrit est le rapport, par l'outil
  d'écriture. `pdftotext` n'écrit que vers la sortie standard (`-`).
- Aucun `git checkout`, `switch`, `reset`, `stash`, `commit`, `push` ; `git diff`, `git show`, `git log` seulement.
- Aucune donnée réelle ; aucune base du NAS.
