# Epic 29 — Préparer la première clôture, et trois défauts de la réconciliation

**Kickoff : 2026-10-14.** La v0.13.3 est en fonction sur le NAS depuis le 2026-10-14. Guy y importe désormais **un
relevé par mois**. L'epic part du choix fait avec Guy à la rétrospective de l'Epic 28 (§ 9 et sa rectification) :
**le bouclement d'exercice ([#232], P1), réduit à ce qui manque**, et trois P2 de la réconciliation ([#529], [#527],
[#651]). **Cibles : v0.13.4** (la réconciliation, dès qu'elle est prête) **puis v0.14.0** (le bouclement).

⚠️ **Contrepartie BMAD des issues** — chaque issue tient le détail, cet epic le périmètre. Il se clôt quand les quatre
issues ci-dessous sont fermées et les deux versions publiées.
*(Complété le 2026-10-14 : un quatrième P2 de la réconciliation, [#668], entre dans l'epic avec la story 29-5 — l'epic
se clôt donc sur **cinq** issues.)*

## Ce que le bouclement n'est plus, et ce qu'il reste à faire

Le texte de [#232] date du 2026-07-09. Depuis, **la clôture existe** : un verrou, posé dans l'ordre des exercices,
réouvrable par un administrateur avec un motif (Epic 14, v0.13.0). **Le report à nouveau existe aussi** : il est
calculé en temps réel depuis l'origine (Story 14-1, v0.8.0 ; manuel utilisateur § *Clôture d'un exercice*). Une
clôture de l'exercice 2026 est donc **déjà possible**. Ce qui manque l'outille et la sécurise :

- des **contrôles préalables** à la clôture, **en lecture seule** : comptes d'attente soldés, transactions bancaires
  en attente, écritures déséquilibrées ou hors exercice, tout ce qu'un comptable vérifie avant de verrouiller ;
- une **aide aux transitoires** (actifs et passifs de régularisation), contre-passés automatiquement à l'ouverture de
  l'exercice suivant ;
- peut-être une **écriture d'affectation du résultat** aux fonds propres — à trancher par la fiche, au regard du report
  calculé : une écriture qui double un calcul fausserait le bilan.

La fiche de la 29-1 **établit au code** ce qui manque réellement, avant toute conception, et découpe selon la règle
de découpage préventif.

## La recette du bouclement : pas avant 2027

**Guy (2026-10-14)** : « je ne testerai les bouclements qu'en début 2027, après avoir complètement sauvegardé la base
de données ». Aucune recette en usage réel ne viendra donc rattraper ce que les revues manquent, comme elle l'a fait
pour #632, #640 et la plupart des epics de recette. Trois conséquences :

1. **Les E2E du bouclement portent la charge de la recette** : parcours complet sur une base de démonstration —
   contrôles, transitoires, clôture, ouverture de l'exercice suivant, contre-passation, réouverture.
2. **Une répétition à blanc est prévue avant 2027** : importer une sauvegarde `.keshbackup` du NAS dans une instance
   séparée, sur la station, et y clore 2026 — le NAS n'est pas touché. La marche à suivre va au manuel
   d'administration.
3. **Les contrôles préalables viennent en premier** : ils ne font que lire, Guy peut les essayer dès leur livraison sur
   ses données de 2026 sans risque, et ils servent en cours d'année.

## Le décompte, et la règle de release

```sh
gh issue list --state open --limit 500 --json number --jq length
gh issue list --state open --limit 500 --json labels \
  --jq '[.[]|select([.labels[].name]|any(.=="bug" or .=="known-failure"))]|length'
```

| Moment | Issues ouvertes | dont bugs |
|---|---|---|
| Tag v0.13.3 (2026-10-14) | **138** | 53 |
| Kickoff de l'Epic 29 (2026-10-14) | **138** | 53 |
| **Plafond à chaque tag** | **moins qu'au tag précédent** | — |

Règle de Guy (2026-10-13) : « l'objectif est de diminuer le nombre d'issues ouvertes : dès qu'il y en a moins, on est
gagnant » — **toutes** les issues, mesurées au tag. Un défaut découvert hors de la story en cours **s'ouvre en issue**
même s'il fait remonter le compte (R11) ; ne jamais fermer un défaut réel pour tenir le chiffre.

## Décisions du Project Lead (2026-10-14)

- **Voie 1** de la rectification : le bouclement réduit à ce qui manque, avec #529, #527 et #651.
- **Recette du bouclement en 2027**, après une sauvegarde complète (ci-dessus).
- **Autonomie complète jusqu'à l'image Docker** (« autonomie complète pour 28 », lu 29 — l'Epic 28 étant clos ; lecture
  annoncée à Guy).

## Les stories

Chaque story tranche ses points en autonomie, au registre `epic-29-choix-autonomes.md`, et suit
`docs/agents-orchestration.md`. **Périmètre minimal dès la fiche** (B3 de la rétrospective 28) : un défaut voisin
s'intègre s'il est mécanique, sinon il s'ouvre. **Recette visuelle** : captures clair et sombre pour tout contrôle
visible. **Une simplification de conception qui touche une transaction est relue sur l'ordre des verrous et la
naissance de la vue** (B5).

### 29-0 — Kickoff (doc seulement)

Ce fichier, le registre de l'epic, la section `epic-29` de `sprint-status.yaml`.

### 29-1 — Préparer la clôture d'un exercice ([#232], P1) — la fonctionnalité de l'epic

Fiche d'abord **d'inventaire** : ce que la clôture et le report font aujourd'hui (routes `fiscal_years`, rapports,
`opening_balances`), ce que #232 demande encore, ce qu'un exercice de personne physique non assujettie exige. Découpage
attendu (à confirmer par la fiche) : **29-1a contrôles préalables** (lecture seule, écran et API) → **29-1b
transitoires et contre-passation** → éventuellement **29-1c affectation du résultat**. Manuels (PDF), i18n quatre
locales, E2E de parcours. La répétition à blanc (manuel d'administration) appartient à la dernière sous-story.
*(Rectifié le 2026-10-14, validation P2 de la 29-1, décision de l'orchestrateur : la répétition à blanc quitte la 29-1b
et devient la **Story 29-7** (doc), écrite en l'exécutant réellement sur la station — C-29-1-23.)*

### 29-2 — Un paiement reçu tard est proposé, ou se rapproche à la main ([#529], P2)

La fenêtre `InvoiceDateWindow` (28-7) est centrée sur la date de facture ; un paiement à 30 jours net en retard n'est
jamais proposé, et l'écran n'offre aucun moyen de choisir la facture. **`kesh-db` touché : gate complet même en
boucle.** Ne pas rouvrir #548 : proposition et acceptation gardent une seule borne.

### 29-3 — La meilleure proposition passe en premier ([#527], P2)

Les candidates triées par score, toutes origines confondues, ou une priorité écrite et dite à l'écran ; manuel
(`user-manual.tex`, « la meilleure »).

### 29-4 — Une saisie concurrente ne passe plus sur un compte qu'on clôt ([#651], P2)

Limite L1 de la 28-4. Sérialiser la clôture avec les écrivains **sans** réintroduire les cycles de verrous que la 28-4
a écartés (rapports de validation P2 et P3 de la 28-4) ; piste : rejeu sur interblocage. **Critère hérité de l'AC7
de la 28-4** : aucune naissance de vue déplacée. Story à risque : validation complète Opus et Sonnet, pas de
simplification sans relecture des verrous.

### 29-5 — À score égal, la facture échue la première passe en tête ([#668], P2)

**Ajoutée le 2026-10-14** par l'orchestrateur, pendant les fiches 29-2 et 29-3 qui l'ont relevée hors de leur
périmètre. `propose_matches` départage deux factures de même score par `invoice_id` : la plus anciennement créée
gagne, même pas encore échue, devant une facture échue. La 29-2 **aggrave** le défaut — sa fenêtre élargie fait entrer
des candidates jusqu'à `F ≥ D − 60` à 30 jours net, et des factures non échues à échéance lointaine — ; l'`ORDER BY …
LIMIT 50` de `find_unpaid_invoices_for_window` peut en outre écarter une facture échue au profit d'anciennes non
échues. Critère : **art. 87 CO** (dette exigible, puis échue la première). Les deux sortent dans la **même v0.13.4**,
la 29-5 mergée avant le tag. **`kesh-db` touché : gate complet même en boucle.** Manuel utilisateur : la règle
d'imputation.

### 29-6 — Une facture qu'un trop-perçu de quelques centimes fait refuser n'est plus proposée ([#665], P2)

*Ajoutée par l'orchestrateur le 2026-10-14, relevée pendant la fiche de la 29-3.* La proposition retient une facture à
± 5 centimes de son reste dû (`AMOUNT_TOLERANCE_HUNDREDTHS`) ; l'acceptation refuse un paiement qui dépasse le reste dû
(`classify_payment`, `RECONCILIATION_OVERPAYMENT`). Même famille que #548 (28-7), sur le montant : une seule borne pour
la proposition et l'acceptation. Voie retenue par la fiche : la proposition écarte le trop-perçu ; aucune écriture
nouvelle (l'absorption sur le compte d'arrondi est écartée, C-29-6-2). **`kesh-db` touché : gate complet même en
boucle.** Manuel (PDF), `api-external.md`, CHANGELOG. *Étendue à la validation P1 (2026-10-14, décision de
l'orchestrateur)* : une ligne « Aucune correspondance » se coche, pour *Rejeter* seulement — sans quoi le contournement
« régler depuis la fiche, puis rejeter la transaction », prescrit par la 29-6 et la 29-2 (C-29-2-8), est impraticable ;
un écran, une clé i18n, un spec E2E.

## Ordre

- **29-1 en parallèle** des trois autres : elle ne touche ni la réconciliation ni les comptes bancaires.
- **29-2, puis 29-3, puis 29-4**, en série : `routes/reconciliation.rs` pour les deux premières, les chemins d'écriture
  des comptes bancaires pour la troisième. Chacune se merge avant que la suivante soit rebasée.
- **29-5 après la 29-3** (2026-10-14) : `kesh-db/repositories/reconciliation.rs` partagé avec la 29-2, phrase d'ordre
  des candidates de `docs/api-external.md` et § du manuel partagés avec la 29-3. Aucune zone commune avec la 29-4 :
  les deux peuvent avancer en parallèle, chacune rebasée sur ce qui est mergé.

- **29-6 après 29-2, 29-3 et 29-5** (ordre de développement 29-2 → 29-3 → 29-5 → 29-6) : même requête de candidates
  que la 29-2, et des phrases que la 29-3 écrit sur #665 et qu'elle retire.
- Intégration **sérielle** sur `main`.

### Releases

- **v0.13.4** dès que 29-1a, 29-2, 29-3, 29-4, 29-5, 29-6, 29-8 et 29-9 sont mergées : Guy les éprouve tout de suite avec ses relevés mensuels.

  29-6, R7)* : Guy les éprouve tout de suite avec ses relevés mensuels.
- **v0.14.0** à la fin de la 29-1 : fonctionnalité nouvelle.

À chaque release : `scripts/prepare-release.sh`, macros des manuels et URL au tag, CHANGELOG daté, recomptes du
`CLAUDE.md`, **recompte des issues ouvertes (moins qu'au tag précédent)**, tag annoté, `release.yml`, vérification
Docker Hub (`/health`). Autorisé d'office pour cet epic.

#### Liste de pré-release de la v0.13.4 — contrôles écrits, à exécuter avant le tag

- ⛔ **29-6 (#665) et 29-8 (#670) mergées.** La 29-2 écrit au manuel et au CHANGELOG « enregistrez le règlement depuis
  la fiche de la facture, puis rejetez la transaction » ; ce geste est **impraticable** tant que la 29-6 n'a pas rendu
  rejetable une ligne sans proposition, et la 29-8 doit le remplacer par « puis rattachez ». Contrôle, au moment du
  tag :
  ```sh
  grep -nF 'rejetez la transaction' docs/manual/fr/user-manual.tex
  P='rejet(ez|ant|er)[^.]{0,30}(transaction|-la)'
  tr '\n' ' ' < docs/manual/fr/user-manual.tex | tr -s ' ' | grep -oE "$P"
  sed -n '/^## \[0.13.4\]/,/^## \[0.13.3\]/p' CHANGELOG.md | grep -oE "$P"
  pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oE "$P"
  ```
  ne doivent plus rendre le conseil de rejet **sans** le rattachement — le motif attrape « rejetez la transaction »
  (`:2029`, CHANGELOG), la variante « en rejetant ensuite la transaction » (`sec:facture-posterieure`) et les
  formes voisines (« rejetez-la », « rejeter cette transaction »), le `.tex` étant lu aplati pour qu'un retour à la
  ligne ne coupe pas la phrase ; le PDF aplati est contrôlé par la même commande. Éprouvé sur l'arbre : le `.tex` rend
  les deux sites, la section 0.13.4 du CHANGELOG un, le PDF les deux. Faute de quoi, pas de tag. *(Ajouté le 2026-10-14, revue
  de code P1 de la 29-2, findings B1 = E2 = A1 — décision de l'orchestrateur ; cf. C-29-2-8.)*
- ⛔ **29-5 (#668) mergée** (départage à score égal, C-29-2-13 de la 29-2).

[#232]: https://github.com/guycorbaz/kesh/issues/232
[#527]: https://github.com/guycorbaz/kesh/issues/527
[#529]: https://github.com/guycorbaz/kesh/issues/529
[#651]: https://github.com/guycorbaz/kesh/issues/651
[#668]: https://github.com/guycorbaz/kesh/issues/668

[#665]: https://github.com/guycorbaz/kesh/issues/665
