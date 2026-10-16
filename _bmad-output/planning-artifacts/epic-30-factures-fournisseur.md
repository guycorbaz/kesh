# Epic 30 — Les factures fournisseur : de la pièce au règlement par la réconciliation

**Kickoff : 2026-10-15.** La v0.13.4 est en fonction sur le NAS depuis le 2026-10-15. Guy y importe un relevé par mois
et veut, pour chaque **débit**, saisir la facture fournisseur correspondante — souvent ancienne et **déjà payée** — puis
laisser la **réconciliation** la régler : **un seul processus**, le même que pour une facture client. La recette de la
v0.13.4 l'a montré impraticable aujourd'hui (constats du 2026-10-15) : un document sans QR-code est refusé à l'import,
et la réconciliation ne propose aucune facture fournisseur pour un débit.

**Cible : v0.14.0**, avec la 29-1b déjà mergée (contre-passation des transitoires, #232).

⚠️ **Contrepartie BMAD des issues** — chaque issue tient le détail, cet epic le périmètre. Il se clôt quand les deux
issues ci-dessous sont fermées et la v0.14.0 publiée.

| Issue | Objet | Ordre |
|---|---|---|
| [#686] | la réconciliation propose les factures fournisseur ouvertes pour un débit, et *Accepter* les règle | 1 |
| [#685] | les documents à traiter, et la création d'une facture fournisseur depuis un document ou sans document | 2 |

## Indispensable à la mise en production

**Guy (2026-10-15)** : « je ferai les différents tests lorsque cette épic sera terminée : ces fonctions sont
indispensables à la mise en production. » Deux conséquences :

1. **Aucune recette intermédiaire** ne rattrapera ce que les revues manquent. Les **E2E de parcours** portent la
   charge : un débit du relevé → la facture fournisseur saisie ouverte (depuis un document avec QR, sans QR, sans
   document) → la proposition → *Accepter* → facture payée et lettrée, transaction rapprochée → *Annuler le
   rapprochement*.
2. **Le parcours complet** se juge en fin d'epic, de bout en bout, sur une base de démonstration, avec captures clair
   et sombre, avant la release.

## Le décompte, et la règle de release

```sh
gh issue list --state open --limit 500 --json number --jq length
gh issue list --state open --limit 500 --json labels \
  --jq '[.[]|select([.labels[].name]|any(.=="bug" or .=="known-failure"))]|length'
```

| Moment | Issues ouvertes | dont bugs |
|---|---|---|
| Tag v0.13.4 (2026-10-15) | **137** | — |
| Kickoff de l'Epic 30 (2026-10-15) | **138** (+#685, +#686, −#232) | 49 |
| **Plafond au tag v0.14.0** | **moins de 137** | — |

Règle de Guy (2026-10-13) : moins d'issues ouvertes qu'au tag précédent, **toutes** issues, mesurées au tag. Un défaut
découvert hors de la story en cours **s'ouvre en issue** même s'il fait remonter le compte ; ne jamais fermer un défaut
réel pour tenir le chiffre. Fermer #686 et #685 donne 136 : la marge est **d'une issue**.

## Décisions du Project Lead (2026-10-15)

- **Contenu et ordre** : #686 puis #685, proposés à Guy et non contestés ; la 29-7, la rétrospective de l'Epic 29 et la
  revue de projet suivent.
- **Autonomie complète** jusqu'à l'image Docker v0.14.0 ; recette par Guy en fin d'epic.
- **Conception de #685 par Guy** (texte de l'issue, réécrit le 2026-10-15) : entrée par le dossier seulement ; une
  facture = un document ; rien n'est créé d'office, ni facture ni fournisseur.

## Les stories

Chaque story tranche ses points en autonomie, au registre `epic-30-choix-autonomes.md`, et suit
`docs/agents-orchestration.md`. **Périmètre minimal dès la fiche** ; un défaut voisin s'intègre s'il est mécanique,
sinon il s'ouvre. **Recette visuelle** : captures clair et sombre pour tout écran touché. **Une simplification qui
touche une transaction se relit sur l'ordre des verrous et la naissance de la vue.** Règle « **un appariement propose,
il ne crée jamais** » (`CLAUDE.md`) : elle gouverne les deux stories.

### 30-0 — Kickoff (doc seulement)

Ce fichier, le registre de l'epic, la section `epic-30` de `sprint-status.yaml`.

### 30-1 — La réconciliation règle les factures fournisseur ([#686], enhancement)

Le symétrique fournisseur de la réconciliation client : pour un **débit** en CHF, proposer les factures fournisseur
**ouvertes** (montant égal au reste dû, référence QR/ISR ou numéro, nom du fournisseur) ; *Accepter* passe le règlement
(débit créanciers / crédit banque), passe la facture **payée** et **lettrée**, rapproche la transaction — ce que fait
déjà le règlement par virement depuis la fiche ; patron batch `{ accepted, failed }` ; *Annuler le rapprochement*
défait le règlement. La fiche **établit au code** ce qui existe des deux côtés (`supplier_invoices::pay_in_tx`,
`accept_one_invoice`, `propose_matches`, la fenêtre de dates des Stories 28-7 et 29-2, le trop-perçu de la 29-6, la
priorité de la 29-3 et de la 29-5, le rattachement de la 29-8) et réutilise ce qui peut l'être plutôt que de le
dupliquer. **Fenêtre de dates** à fixer : la facture peut être bien antérieure au paiement, et l'usage de Guy saisit la
facture *après* le paiement. **Voisin** : [#671] (rattacher un débit à un règlement fournisseur déjà saisi depuis la
fiche) — à intégrer seulement si le symétrique de la 29-8 en sort mécanique ; sinon il reste ouvert. **`kesh-db`
touché : gate complet même en boucle.** Manuel utilisateur (PDF), `api-external.md`, i18n quatre locales.

### 30-2 — Des documents à traiter à la facture fournisseur ([#685], enhancement)

Parcours de l'issue : liste des documents du dossier d'entrée, **avec ou sans QR-code**, prévisualisation, suppression
d'un non-facture ; *Créer une facture* depuis un document — document visible pendant la saisie, pré-remplissage par le
QR s'il y en a un, fournisseur **proposé** ou **créé sans quitter** la saisie ; le document quitte la liste et le
dossier, reste consultable depuis la facture et son écriture ; facture sans document ; document joint après coup.
**Découpage attendu** (règle de découpage préventif : la story touche l'import, le stockage des pièces, les factures
fournisseur, les contacts et plusieurs écrans) — la fiche établit l'existant (import du dossier `KESH_INBOX_DIR`, liste
« à compléter », `KESH_DOCUMENTS_DIR`, saisie manuelle) puis découpe, par exemple : **30-2a** la liste accepte les
documents sans QR, prévisualisation, suppression → **30-2b** la création côte à côte, fournisseur proposé ou créé en
ligne → **30-2c** document joint après coup et consultable depuis l'écriture. La sauvegarde doit emporter les pièces
([#503]) : la fiche vérifie ce qu'il en est et le dit. Manuel utilisateur (PDF), i18n quatre locales, E2E de parcours.

### 30-3 — Le chemin de la facturation ([#585], bug P4)

Petite story de texte : le manuel et les messages écrivent « Paramètres → Facturation » ; le vrai chemin est
« Administration → Facturation ». Aucun changement de navigation. Fiche : `30-3-chemin-facturation.md`.

## Hors périmètre

Plusieurs pages séparées pour une même facture (fusionner les PDF avant dépôt) ; dépôt par le navigateur ; tickets de
caisse ([#254]) ; import automatique du dossier ([#459]) ; factures client (pas de document externe joint, pas d'état
« envoyée » — constats de la recette v0.13.4, non demandés).

[#686]: https://github.com/guycorbaz/kesh/issues/686
[#685]: https://github.com/guycorbaz/kesh/issues/685
[#671]: https://github.com/guycorbaz/kesh/issues/671
[#585]: https://github.com/guycorbaz/kesh/issues/585
[#503]: https://github.com/guycorbaz/kesh/issues/503
[#254]: https://github.com/guycorbaz/kesh/issues/254
[#459]: https://github.com/guycorbaz/kesh/issues/459
