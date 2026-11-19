# Registre des choix autonomes — story 48-15 (clé `e4815`)

Story : `48-15-troncatures-silencieuses` (#1331, #1332, #1333, #1334). Agent de développement Opus 5.5, 2026-11-18.
Chaque choix : contexte, option retenue, autres options, réversibilité.

## C-e4815-1 — L'assertion de présence de la 48-13 sur l'e-mail devient une assertion d'absence

- **Contexte** : la fiche (V4-2, AC8) dit que la 48-13 « n'écrit aucun test de présence » du `maxlength={320}` de l'e-mail. À la
  tête de la branche (`45eb99ddd`), le test de la 48-13 `AC4 — aucun des cinq champs de l'adresse, ni le téléphone ni le site web
  n'a de maxlength ; l'e-mail garde le sien (M4)` (`settings-page.test.ts:1562-1575`) l'asserte : `expect(getByTestId(
  'settings-company-email-input').getAttribute('maxlength')).toBe('320')`. Retirer l'attribut le fait rougir.
- **Retenu** : retirer ces deux lignes (et la fin du titre) du test de la 48-13, avec un commentaire qui renvoie à la 48-15 ; écrire
  l'assertion d'absence dans un bloc propre à la 48-15 (`e-mail de la société : aucun maxlength`, M19).
- **Écarté** : réécrire le test de la 48-13 en place (son titre et sa mutation M4 restent les siens) ; garder la présence (contraire
  à D2).
- **Réversibilité** : totale (un test).

## C-e4815-2 — Les tests de l'assistant acceptant 70 : deux tests, une base chacun

- **Contexte** : AC3 demande « un prénom seul de 70 et un nom seul de 70 acceptés ». L'acceptation franchit l'étape 5 : un second
  cas sur la même base rendrait `ONBOARDING_STEP_ALREADY_COMPLETED`.
- **Retenu** : une aide `assert_lone_70_accepted` et deux tests `#[sqlx::test]`, chacun sur sa base (patron de la 48-13,
  `assert_name_refused`, « un cas par test, chacun sur sa base »).
- **Écarté** : vider les tables entre deux cas (fragile : clés étrangères).
- **Réversibilité** : totale.

## C-e4815-3 — Le délai de paiement de 1000 jours : filet écrit dans le fichier neuf

- **Contexte** : AC7 laisse le choix entre citer `contact_payment_terms_e2e.rs` (`contact_days_out_of_bounds_returns_400`, cas
  `[-1, 366]`) et y ajouter 1000.
- **Retenu** : le cas 1000 (et 365 accepté) dans `pdf_fields_are_refused_past_their_bound_and_accepted_at_it`
  (`contact_field_lengths_e2e.rs`), à côté des autres champs du PDF, avec lecture de `error.code` ; le test existant reste cité.
- **Écarté** : modifier `contact_payment_terms_e2e.rs` (hors de la liste des fichiers de la fiche).
- **Réversibilité** : totale.

## C-e4815-4 — Le motif d'un refus de l'écran des contacts s'affiche sous le champ (raison sociale, délai)

- **Contexte** : deux des champs du PDF sont jugés **aussi par l'écran des contacts** : la raison sociale
  (`formName.trim().length > 255`, `contacts/+page.svelte:452-453` à `45eb99ddd`) et le délai de paiement (0 à 365, `:459-468`).
  Leur refus rend `formValidation` non vide, ce qui **désactive** le bouton (`:1132`, `disabled={formSubmitting ||
  !!formValidation}`) ; le motif n'est affiché qu'à la soumission (`formError = formValidation`, `:477`), qu'un bouton désactivé
  n'atteint jamais. Le `maxlength` rendait ce cas inatteignable (le texte collé était coupé). Le retirer sans plus ferait passer d'une
  coupure muette à un **refus muet** (bouton grisé sans explication), contraire à la story (« refusé en le disant ») et à l'AC7 /
  l'AC12 (6), qui attendent le message « entre 0 et 365 jours ». **La page des articles n'est pas concernée** : elle affiche déjà le
  motif au pied du formulaire dès la frappe (`products/+page.svelte:737-738` au commit `865e54132`, `{:else if formTouched &&
  formValidation}`) ; le premier ajout y faisait doublon, vu à la capture `08`, retiré au commit de la recette.
- **Retenu** : les deux messages deviennent des valeurs dérivées (`nameTooLongMsg`, `paymentTermsDaysMsg`), lues par
  `formValidation` à la même place qu'avant (ordre et texte inchangés) **et** affichées sous leur champ, en `text-destructive`, avec
  un `data-testid` (`contact-name-too-long`, `contact-payment-terms-days-error`). Aucune clé neuve, aucun site `i18nMsg` de plus
  (l'appel se déplace).
- **Écarté** : ne rien afficher (refus muet) ; réactiver le bouton pour laisser `submitForm` afficher `formError` (change le
  comportement de toute la validation de l'écran, hors périmètre) ; un message neuf par champ (clé neuve, quatre catalogues).
- **Réversibilité** : totale (écran seul).
- **Amendement — revue P1 (B1 = E1 = A1, MEDIUM ; décision de l'orchestrateur du 2026-11-19)** : « ordre et texte inchangés » ne
  tenait pas. Une raison sociale de plus de 255 caractères, collée en type « Entreprise » puis laissée par une bascule vers
  « Personne » (`formName` n'est remis à zéro qu'à l'ouverture), grisait le bouton **sans motif visible** : le champ et son motif
  n'existent qu'en « Entreprise ». Le refus était de plus sans objet : pour une personne, le serveur recompose le nom depuis le
  prénom et le nom (`contacts.rs`, branche `ContactType::Personne`). **Désormais, la raison sociale ne se juge qu'en type
  « Entreprise »** (`formContactType === 'Entreprise' && …`) : `formValidation` change donc pour une personne (la longueur de
  `formName` n'y compte plus), ordre et texte restant ceux d'avant pour une entreprise. Dans le même correctif : la longueur se
  compte en **caractères** (`[...formName.trim()].length`, comme `chars().count()` du serveur ; B2, E2), à la page des contacts et à
  celle des articles ; le délai n'accepte que des **chiffres** (`/^\d+$/` avant `Number`, qui lisait « 0x1E » comme 30 ; B5, E5).
  **Écarté** : vider `formName` à la bascule (perd la saisie si l'utilisateur revient en « Entreprise ») ; afficher le motif hors
  des deux branches (garde un refus sans objet pour une personne).
