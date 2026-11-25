# Prompt — validation P4 de la fiche 48-12b « Doublons de factures fournisseur — écran »

Passe **complète** (non ciblée), **modèle Sonnet** (la P3 était Opus), **contexte frais**, **lecture seule**. Réponds en français soigné.

## Ce que tu lis

- Worktree : `/home/gcorbaz/devel/kesh-48-12` (branche de la story 48-12). Lis d'abord son `CLAUDE.md` (§ *Review Iteration Rule*, § *Propagation post-patch*, § *Inventorier les sites NON RÉSOLUS*, § *Recette visuelle*, § *Issue Tracking Rule*), puis `/home/gcorbaz/devel/kesh-gate-logs/prompts/e0200-commun.md` et ce qu'il nomme.
- Fiche à valider : `_bmad-output/implementation-artifacts/48-12b-doublons-fournisseurs-ecran.md`.
- Fiche sœur, **à lire en entier** (contrat commun) : `_bmad-output/implementation-artifacts/48-12a-doublons-fournisseurs-serveur.md`. Fiche-mère : `48-12-reponses-perdues-fournisseurs.md`.
- Rapports précédents : `/home/gcorbaz/devel/kesh-gate-logs/e4812/validation-12b-p1.md`, `validation-12b-p2.md`, `validation-12b-p3.md` (et ceux de la 48-12a à côté).
- La remédiation de la P3 : le dernier commit de la branche (`git show HEAD`). Le code n'a pas changé depuis `af4e0e313` : vérifie-le (`git diff --stat af4e0e313 HEAD`).

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

1. **Les remédiations de la P3, une par une** (V3-1 à V3-12 de `validation-12b-p3.md`) : prémisse relue au code, puis fermée / fermée en partie / déplacée.
2. **Décisions de Guy du 2026-11-19 sur E6** : première (formulaire figé, ni confirmation ni renvoi, vérifier dans la liste) et seconde (bouton « J'ai vérifié, elle n'existe pas — reprendre la saisie » → E1, saisie intacte, **sans rien envoyer** ; variante « modification » : texte et libellé adaptés, lien vers la fiche). Vérifie que la reprise n'est ni une confirmation ni un renvoi ; que toutes les entrées de E6 y mènent ; que la conséquence (renvoi d'une facture qui existait → E4/E5 ; modification déjà écrite → `OPTIMISTIC_LOCK_CONFLICT`) est exacte ; que l'AC-8 et l'encart du manuel `:2362-2370` sont alignés.
3. **Relevé C — ensemble clos des sites qui écrivent la saisie de façon asynchrone** : refais-le toi-même **depuis le symptôme** (`await`, `$effect`, `.then`, `setTimeout`, rappels, props que la page réaffecte) dans `SupplierInvoiceForm.svelte`, `SupplierCreatePanel.svelte` et les pages qui montent le formulaire (`new/+page.svelte`, `[id]/edit/+page.svelte`). Un site absent, mal résolu ou un angle mort non justifié est un finding. La résolution (« Enregistrer » inactif pendant un scan ou panneau ouvert, écritures ignorées hors E1/E4/E7, `$effect` gardé) ferme-t-elle la divergence entre saisie montrée et corps gardé, en E5 comme en E6 ? change-t-elle un comportement existant testé ?
4. **Relevé D — ensemble clos des gestes et relectures de la fiche d'une facture annulée** : refais-le depuis le symptôme (`onclick`, conditions d'affichage par statut `[id]/+page.svelte:851-1211`, toutes les affectations de `invoice`, `rereadAfterUnknownOutcome`, `reloadAfterRefusal`, `reloadAfterStaleRefusal`, boîtes de confirmation). Chaque site doit effacer l'avis F4 ou le garder avec raison. Lis le corps de `attach`, `detach`, `reloadAfterRefusal`, `clearRefusals`, et dis si les sites « neufs » suffisent ; un site manquant est un finding.
5. **La table d'états, complète** : reconstruis la matrice **chaque état × chaque contrôle** des tables A (E1 à E9) et B (F1 à F6), « Retenir », « C'est la bonne date », bouton de reprise de E6 et `$effect` compris ; deux gardes synchrones (`submit` en E1/E4/E7, confirmation en E5) ; transitions manquantes, cases muettes, contradictions, états inatteignables ou sans sortie non voulue.
6. **Chaque ligne a un AC et un test, chaque AC une ligne ou la mention « transverse »** ; chaque mutation est tuée par le test de sa ligne ; les fichiers de test nommés existent (sinon la fiche le dit).
7. **Un seul texte par refus, pour les deux fiches** : l'écran affiche les clés `error-supplier-invoice-*` de la 48-12a (D7), mot pour mot ; vérifie `frontend/scripts/lint-i18n-ownership.js` (`GLOBAL_NAMESPACES`) et que le frontend lit bien ces catalogues (`crates/kesh-i18n/locales/`).
8. **Cohérence entre les deux fiches (contrat d'API commun)** : codes, `details`, `confirmable`, `confirmNotDuplicate` sur les quatre corps, textes, règle de sortie commune ; relevé B de la 48-12a contre les sites que la 48-12b dit écrire.
9. **#985 (D2, D2bis, E8, E9)** : ordre `to_complete` puis `completed`, filtre et `try/catch` d'`onRefusal`, `locked`, dette nommée, angle mort du détachement.
10. **Manuel contre le code** : `docs/manual/fr/user-manual.tex` (sites cités : `:2067-2087`, `:2195-2212`, `:2242-2265`, `:2293-2312`, `:2362-2370`, `:2682-2721`) ; **PDF compris**, aplati avant de greper (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`, en pipe) ; numéros de ligne ; contrôles positifs d'AC-9.
11. **Propagation** : grep des symptômes corrigés en P3 (« figés de E2 à E5 », « lecture de la décision », « aucune dans la page : quitter ou recharger », « il est un état de `clearRefusals` », « quatre modules », « pour la supprimer ou la corriger ») sur les deux fiches et la fiche-mère ; seules les entrées historiques du Change Log peuvent les garder.
12. **Modules, recyclage, recette visuelle, usage réel** : cinq modules (`SupplierCreatePanel.svelte` non modifié ?) ; un défaut de la P3 revient-il sous une autre forme, né de cette remédiation ? la liste des captures couvre-t-elle chaque état neuf ou modifié ? un chemin mène-t-il encore à un second brouillon par un renvoi à l'aveugle, ou à une saisie perdue sans que l'écran l'ait dit ?

## Interdits

Lecture seule. N'exécute **aucune** commande qui écrit ou construit : pas de `cargo`, pas de `npm` ni `npx`, pas de `make`, aucun `scripts/*` (dont `scripts/prepare-release.sh`, `scripts/test-fast.sh`, `scripts/gate-ci.sh`, `scripts/mem-guard.sh`), pas de `docker`, pas de `mariadb`, aucun `git` qui écrit (`commit`, `checkout`, `switch`, `reset`, `stash`, `add`, `push`, `worktree`, `branch -D`…), pas de `gh`, **aucune redirection** (`>`, `>>`, `tee`) ni fichier temporaire, aucune écriture sous `~/kesh/`. Lis par `cat`, `sed -n`, `grep`, `git show`, `git log`, `git diff`, `git grep`, `pdftotext … -` en pipe.

Le seul fichier que tu écris est ton rapport : **`/home/gcorbaz/devel/kesh-gate-logs/e4812/validation-12b-p4.md`**.

## Rapport

- En tête : passe, modèle, commit lu, fichiers lus, fichier écrit.
- **Verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW, recompté depuis la liste des findings, total cohérent avec la ventilation.
- Chaque finding : identifiant `V4-n`, sévérité, axe, lieu dans la fiche (ligne de table comprise), **code lu** (ligne citée), preuve, remède.
- La matrice état × contrôle de l'axe 5, telle que tu l'as reconstruite (cases muettes marquées).
- Vérifications positives.
- **Axes exercés** et **axes non exercés ou partiels**, chacun avec sa raison : un « 0 finding » sans cette liste ne compte pas.
- Signal de recyclage : déclaré ou non, avec le constat.
- Pour les relevés que tu as refaits (A et B pour la 48-12a ; C et D pour la 48-12b) : ton propre ensemble, ligne par ligne, comparé à celui de la fiche.
