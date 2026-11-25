# Prompt — validation P7 de la fiche 55-2 (ciblée, sur les décisions du mainteneur)

## Contexte

Kesh : comptabilité suisse (Rust/Axum, MariaDB, SvelteKit), en production chez son mainteneur. Epic 55, « Navigation de
l'interface » : `_bmad-output/planning-artifacts/epic-55-navigation-interface.md` (il fait foi pour le périmètre et les
décisions D1 à D14). Worktree `/home/beat/claude/kesh/.claude/worktrees/kesh-e55`, branche `epic/55-navigation`, partie de
`origin/main` à `1e23edb55`. Le développement repartira de la PR de l'Epic 46, non fusionnée : lis-la par
`git show pr/46-rapports-groupes:<fichier>` (`511e61d54`) ; l'Epic 47 par `git show epic/47-comptes-miroirs:<fichier>`.

Passe **P7**, ciblée, **contexte frais**, modèle Opus. Langue du rapport : français. La validation en est à P6 (0/0/1/5, renvoi au développement R-1).

## À lire d'abord

1. `CLAUDE.md` du worktree : § *Review Iteration Rule* et ses sous-sections, § *Test Locally First*, § *Travailler à
   plusieurs développeurs*, § *Issue Tracking Rule*, § *Priorités des défauts*.
2. Le fichier d'epic en entier.
3. La fiche : `_bmad-output/implementation-artifacts/55-2-lien-de-compte-partout.md`.
4. Le code qu'elle cite, **ligne par ligne** : chaque `chemin:ligne` se vérifie à la base indiquée.

**Les décisions du mainteneur à verser** (2026-11-25 ; c'est contre elles que la passe juge la fidélité) :
1. **Q1 — acceptée avec conditions** : `hasDocuments` et `GET /journal-entries/{id}/documents` tels quels ; l'aperçu
   `GET /invoices/{id}/pdf/preview` accepté si le PDF **porte lui-même** la mention de Q10 sur chaque page ; facture **déjà
   figée** : l'aperçu montre le PDF figé **tel quel, sans filigrane** ; l'aperçu non figé ne sert qu'aux factures
   **validées dont le PDF n'est jamais sorti**.
2. **Q10 — dans le PDF d'aperçu non figé** : filigrane « APERÇU » en diagonale, assez gros, pouvant passer sur le texte
   mais **nettement moins foncé** que lui, la facture restant lisible même imprimée en noir et blanc ; en en-tête
   « Aperçu — non émis » ; **QR-facture barrée et non payable** (section de paiement remplacée par un cadre barré) ; **un
   seul mécanisme de rendu** pour « APERÇU » et « BROUILLON » ; recette visuelle avec une facture de plusieurs lignes ;
   mécanisme dans la génération des PDF (à vérifier au code) ; la 55-3a s'agrandit, sa taille réévaluée, signalée sans
   découpage (epic à six stories).
3. **Q3 — oui** : lien TVA par les comptes désignés des paramètres de la TVA ; sans compte désigné, pas de lien.
4. **Q5 — oui** : issue des justificatifs en P4, `enhancement`.
5. **Q6 — dérogation partielle**, section « Dérogation » du fichier d'epic : stories purement écran → tests des Rapports et
   de la navigation à chaque story, E2E complet et `gate-ci.sh` en fin d'epic avant la PR vers `main` ; la story des
   justificatifs qui touche le serveur → gate complet et E2E complet à sa clôture ; E2E de fin d'epic rouge → l'epic
   n'entre pas dans `main` avant que la story fautive soit trouvée et corrigée.
6. **Q8 — non** : la fiche de la facture garde son bouton, qui fige au premier téléchargement (geste d'émission) ; écrit
   comme décision.
7. **Q9 — oui, epic à part** (numéro à confirmer) pour les liens de compte hors des Rapports, dépendant de l'Epic 55, qui
   pose le composant de lien et le saut avec la ligne surlignée ; vérifier que l'Epic 55 les pose de façon réutilisable, le
   dire au « hors périmètre ».
8. **PDF produit hors de Kesh** (évolution, non livrée) : le PDF fourni **est** la pièce émise (copié, sauvegardé, figé,
   jamais régénéré) ; ni second PDF ni QR-facture de Kesh ; « lien seulement » sur demande seulement (pas dans le
   `.keshbackup`).
9. **`main` a avancé** : la #1582 (`fbfa64a27`, absente du worktree) touche `frontend/src/routes/(app)/+layout.svelte`,
   l'en-tête et le titre de l'onglet ; à ajouter aux fichiers communs si l'Epic 55 touche ce fichier ou le titre.
10. **PDF du brouillon** : **pas** un livrable de l'Epic 55 ; la 55-3a dit seulement que le mécanisme doit pouvoir le servir ;
    question ouverte au fichier d'epic « PDF du brouillon : dans l'Epic 55 ou à part ? ».
Les questions tranchées sont retirées de la section *Questions au mainteneur*.

**Passe ciblée** : une seule lentille, braquée sur le commit `e0549d45a` (`git show e0549d45a`), qui verse ces décisions.
Juge la 55-2 seulement (en-tête, AC-6, § *Gates*, C-55-2-4, Change Log), contre le fichier d'epic (*Hors périmètre*, D7, *Dérogation*). Au-delà de la sixième passe : ses points iront au T0 (R-n), sans remédiation.
Vérifie que chaque changement est juste au code, fidèle aux décisions (ni plus, ni moins), sans défaut neuf, et propagé
(même symptôme ailleurs dans la fiche, le fichier d'epic et les autres fiches 55-* : `grep` de la valeur, pas de la
formulation). Les axes ne s'exercent que sur ce que touche ce commit.

## Axes (chacun exercé, ou déclaré non exercé avec sa raison)

- **A1 — Prémisses relues au code** : chaque affirmation sur le code (chemin, ligne, comportement) est-elle juste ?
  Recopie la ligne lue pour chaque point contrôlé.
- **A2 — Fidélité aux décisions** : chaque décision versée l'est-elle exactement, sans ajout ni oubli ; une question
  tranchée reste-t-elle écrite comme ouverte quelque part ?
- **A3 — Critères constatables** : chaque AC neuf ou modifié dit-il comment on le constate (test nommé, commande, valeur
  attendue) ? Un AC invérifiable, ambigu ou vrai par construction est un finding.
- **A4 — Cas limites et modes d'échec** introduits par le commit.
- **A5 — Preuve du rouge** des tests neufs : chacun rougit-il sans le correctif ? Un test muet possible ?
- **A6 — Manuel** : une phrase de `docs/manual/fr/user-manual.tex` (et du PDF, `pdftotext docs/manual/fr/user-manual.pdf - |
  tr '\n' ' ' | tr -s ' '`) que la story rendra fausse sans que la fiche le dise ?
- **A7 — Règles du projet** : DRY, i18n quatre locales, captures, splitting préventif (> 5 modules), note de version
  (aucun numéro d'epic, `refs #`), aucun mot-clé de fermeture, dérogation écrite comme telle.
- **A8 — Q9** : `AccountLink` (AC-1) est-il réutilisable hors des Rapports comme C-55-2-4 le dit ? La ligne surlignée est-elle bien absente de la fiche ?
Sévérités : CRITICAL / HIGH / MEDIUM réservées à ce qui fausserait le développement, la vérification ou le produit ;
LOW pour la rédaction sans effet. Ne gonfle pas, ne minimise pas.

## Interdits

Lecture seule. N'écris **aucun** fichier du dépôt ni du worktree. Aucun `cargo`, `npm`, `make`, `docker`, `mariadb`,
`sqlx`, aucun `scripts/*` (en particulier ni `scripts/prepare-release.sh`, ni `scripts/test-fast.sh`, ni
`scripts/gate-ci.sh`, ni `scripts/mem-guard.sh`). Aucun `git commit`, `push`, `fetch`, `checkout`, `switch`, `stash`,
`merge`, `reset`, `worktree`. **Aucun `gh`.** Ne touche à aucun port ni service. `grep`, `git show`, `git log`, `sed -n`,
`pdftotext`, `python3 -I` en lecture sont admis.

## Rapport

Écris **un seul fichier** : `/home/beat/claude/kesh-gate-logs/s55-decisions/55-2-validation-p7.md`, avec :
- le **verdict chiffré** : CRITICAL / HIGH / MEDIUM / LOW ;
- chaque finding : identifiant, sévérité, ce qui est faux, la preuve (ligne lue), le correctif proposé ;
- la **liste des axes exercés et non exercés** (une passe qui ne la donne pas ne compte pas).
Puis rends ce même contenu en réponse, en entier.
