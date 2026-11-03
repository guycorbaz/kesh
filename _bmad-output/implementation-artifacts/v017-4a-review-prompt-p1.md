# Revue de code P1 — Story v017-4a (facture fournisseur en brouillon, backend) — #849

*Prompt versionné (`CLAUDE.md` § *La passe ciblée* : une passe se rejoue et se conteste). Trois lentilles, **B**
(Blind Hunter), **E** (Edge Case Hunter), **A** (Acceptance Auditor), chacune en **contexte frais**, modèle **Opus**.
Langue : français soigné.*

## Le périmètre

- Worktree : `/home/gcorbaz/devel/kesh-0175`, branche `story/v017-4-facture-fournisseur-brouillon`.
- Le diff à relire : `git diff 8e126cfd..HEAD` **hors** `_bmad-output/` pour le code ; les fiches à lire en entier :
  `_bmad-output/implementation-artifacts/v017-4-facture-fournisseur-brouillon.md` (parente : contexte, contrat,
  verrous, précédence, inventaire clos, AC-1 à AC-20 et le début de l'AC-30), `v017-4a-facture-fournisseur-brouillon-backend.md`
  (sous-fiche et Dev Agent Record), `v017-4-choix-autonomes.md` (C-v017-4-1 à 51).
- Lentille **B** : le diff seul, sans les fiches (sauf pour situer un fichier) — défauts de code, de verrous, de
  transactions, d'erreurs, de doc-comments faux.
- Lentille **E** : chaque branche et chaque borne du code neuf — statuts, versions, documents (complétion, joint, lien
  antérieur), lots (`generated`, annulé, confirmé), rapprochements, exercices, périodes, échéances, filtres.
- Lentille **A** : chaque AC (1 à 20, et le début de l'AC-30), chaque mutation M1 à M20 (M12a, M12c, M20-bis à
  M20-quater), le Dev Agent Record **contre ce qui a tourné** (journaux de `/home/gcorbaz/devel/kesh-gate-logs/v0174a/`).

## Les axes obligatoires (chaque lentille, sous son angle)

1. **Migration et données existantes.** `crates/kesh-db/migrations/20261031000001_supplier_invoices_draft.sql` :
   breaking au sens de P1/P3, `min_required` reposé à `0.17.0` en dernière instruction, aucune écriture d'une table
   applicative, jamais un état sans garde entre deux instructions ; P5 (ligne et compteurs de
   `docs/migrations-idempotence-audit.md`, recomptés depuis le tableau), P6 (`migrations_upgrade_path.rs`, `total` et
   `total - N`, copies triées), P7 (`EXEMPT_MIGRATIONS`, compteur), P8 (`migrations.sha384`, aucune migration publiée
   modifiée — `git diff 8e126cfd..HEAD -- crates/kesh-db/migrations/` ne doit montrer qu'un fichier **ajouté**), squash
   régénéré. Les tests de montée (AC-2) prouvent-ils vraiment une relecture **à l'identique** (colonnes fixées avant la
   montée, toutes les tables touchées) ? Les sauvegardes sans `link_origin` (AC-3) s'importent-elles par le chemin
   « colonne absente de la source » ? Le démarrage réel sur une base rc1 peuplée (Dev Agent Record) est-il crédible ?
2. **Verrous et rejeu.** Pour chaque route neuve ou changée (§ *Verrous et rejeu* de la parente) : ordre réel des
   verrous dans le code, première lecture non verrouillante, enveloppe de rejeu présente et du bon type (`DbError` /
   `AppError`), registre `audit_route_registry.rs` (classement, points (ii) et (vi)). Le `DELETE` prend-il la facture
   **avant** le document ? Les tests pilotés (M20, M20-quater) prouvent-ils ce qu'ils disent, sans privilège
   `PROCESS` ? Le test de route du `DELETE` est-il bien présenté comme une non-régression sans vertu de preuve ?
3. **Pattern batch** (`CLAUDE.md`). Un brouillon dans `POST /payment-batches` et dans `POST /reconciliation/accept` :
   refus **par élément** dans `failed[]`, `200`, jamais un `500` ni une `AppError` globale ; décodage tolérant de
   `payment_batches.rs` (l'écriture extraite **après** le statut).
4. **`api-external.md`.** La section « Le cycle d'une facture fournisseur », le tableau des codes, § *Interblocages*,
   les phrases `:81`, `:837`, `:839`, `sourceDocument`, la complétion, les contrôles de clôture (huit) : chaque
   affirmation contre le code. Règle Q4 : **aucun** texte neuf ou modifié ne nomme l'état `open` « ouverte »
   (`grep -n -i 'ouvert' docs/api-external.md`, trié à la main).

Et, comme toute passe sur du code visible (`CLAUDE.md` § *Le prompt d'une passe doit NOMMER le manuel*) : le **manuel
d'administration** (`docs/manual/fr/admin-manual.tex`) **et son PDF** (`docs/manual/fr/admin-manual.pdf`), lu aplati —
`pdftotext docs/manual/fr/admin-manual.pdf - | tr '\n' ' ' | tr -s ' '` — : § *Passer à la 0.17.0* (deux migrations),
« huit contrôles », la validation au lieu de « l'enregistrement » (comptes créanciers et impôt préalable), le compte
du registre (« 135 des 142 », « 135 + 4 + 3 = 142 »), la dévalidation d'une facture fournisseur par une clé
`read-write`. Et le CHANGELOG, section `[0.17.0] — Non publié` (en-tête, *Modifié*).

## Le rapport

- Lentille B : `/home/gcorbaz/devel/kesh-gate-logs/v0174a-review-p1-B.md` ; E : `…-E.md` ; A : `…-A.md`.
- Chaque finding : sévérité (CRITICAL, HIGH, MEDIUM, LOW), lieu (`fichier:ligne`), **ligne lue citée**, défaut, remède.
  Toute affirmation de présence ou d'absence s'appuie sur une sortie `grep -nF` copiée.
- **Axes exercés et axes non exercés**, listés : un « 0 finding » sans cette liste ne compte pas.
- Décompte final recompté depuis la liste.

## Les interdits — lecture seule stricte

- Aucune commande qui écrit, compile ou exécute : **ni `cargo`, ni `npm`, ni `make`, ni `python3`, ni `docker`, ni
  `curl`, ni `claude`**, ni aucun script du dépôt (`scripts/*`, dont `scripts/prepare-release.sh`), ni `sed -i`, ni
  `rm`, `pkill`, `killall`.
- **Aucune redirection** (`>`, `>>`, `tee`), aucun fichier temporaire, aucun fichier dans le scratchpad : un fichier
  créé par erreur se déclare au rapport, il ne se supprime pas. Le seul fichier écrit est le rapport, par l'outil
  d'écriture.
- Aucun `git checkout`, `switch`, `reset`, `stash`, `commit`, `push` ; `git diff`, `git show`, `git log` seulement.
- Aucune donnée réelle ; aucune base du NAS.
