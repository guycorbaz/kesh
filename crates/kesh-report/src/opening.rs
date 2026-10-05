//! La règle du solde d'ouverture, partagée par le grand livre et la balance des
//! comptes (Story 25-5-b, #385).
//!
//! # La règle
//!
//! Elle **diffère selon le type de compte** :
//!
//! - `Asset` / `Liability` (comptes de bilan) : cumul sur `entry_date < début`,
//!   **tous exercices confondus** — même patron que le bilan.
//! - `Revenue` / `Expense` (comptes de résultat) : cumul depuis le **début de
//!   l'exercice** contenant le début de la période seulement. Si la période
//!   commence le premier jour d'un exercice, l'ouverture vaut 0.
//!
//! Un compte de bilan reporte son solde d'un exercice à l'autre ; un compte de
//! résultat est soldé au bouclement et repart de zéro.
//!
//! ⚠️ **Kesh ne passe aucune écriture de clôture** : cette remise à zéro n'existe
//! que comme borne basse d'une somme. Les appelants lisent donc, en SQL, **deux
//! paires de sommes brutes** — avant le début, et depuis le début de l'exercice —
//! et c'est [`opening_from`] qui choisit, **ici seulement**. Recopier ce choix en
//! `CASE` SQL dans un rapport en ferait une seconde règle, qui divergerait en
//! silence.
//!
//! # Limite connue
//!
//! Le signe est lu sur le `account_type` **courant** du compte : retyper un
//! compte mouvementé re-signe tout son historique (issues #274 et #382).

use kesh_db::entities::AccountType;
use rust_decimal::Decimal;

/// Une paire de sommes brutes `(débit, crédit)`, telle que la lit SQL.
pub type DebitCredit = (Decimal, Decimal);

/// `true` si le compte a le débit pour côté naturel.
pub fn is_debit_natured(t: AccountType) -> bool {
    matches!(t, AccountType::Asset | AccountType::Expense)
}

/// `true` pour un compte de bilan — dont le solde se reporte d'un exercice à
/// l'autre.
pub fn is_bilan(t: AccountType) -> bool {
    matches!(t, AccountType::Asset | AccountType::Liability)
}

/// Applique la convention de signe du dépôt — celle de la balance, du bilan et
/// du grand livre : positif du côté naturel du compte.
pub fn signed(t: AccountType, debit: Decimal, credit: Decimal) -> Decimal {
    if is_debit_natured(t) {
        debit - credit
    } else {
        credit - debit
    }
}

/// Ramène un solde **signé par type** en sens débit (débit − crédit), quel que
/// soit le compte. C'est la grandeur qui s'additionne d'un compte à l'autre :
/// additionner des soldes signés par type sans les re-signer mélange les deux
/// conventions.
pub fn debit_sense(t: AccountType, signed_balance: Decimal) -> Decimal {
    if is_debit_natured(t) {
        signed_balance
    } else {
        -signed_balance
    }
}

/// Solde d'ouverture, signé par type, depuis les deux paires de sommes brutes :
/// `before_start` (`entry_date < début`) pour un compte de bilan,
/// `since_fy_start` (`entry_date ∈ [début de l'exercice, début[`) pour un compte
/// de résultat.
///
/// Un appelant dont la période n'est couverte par aucun exercice passe une paire
/// `since_fy_start` **nulle** : l'ouverture d'un compte de résultat vaut alors 0.
pub fn opening_from(
    t: AccountType,
    before_start: DebitCredit,
    since_fy_start: DebitCredit,
) -> Decimal {
    let (debit, credit) = if is_bilan(t) {
        before_start
    } else {
        since_fy_start
    };
    signed(t, debit, credit)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(v: i64) -> Decimal {
        Decimal::from(v)
    }

    #[test]
    fn signe_suit_la_nature_du_compte() {
        assert_eq!(signed(AccountType::Asset, d(100), d(30)), d(70));
        assert_eq!(signed(AccountType::Expense, d(100), d(30)), d(70));
        assert_eq!(signed(AccountType::Liability, d(30), d(100)), d(70));
        assert_eq!(signed(AccountType::Revenue, d(30), d(100)), d(70));
    }

    #[test]
    fn seuls_les_comptes_de_bilan_reportent_leur_solde() {
        assert!(is_bilan(AccountType::Asset));
        assert!(is_bilan(AccountType::Liability));
        assert!(!is_bilan(AccountType::Revenue));
        assert!(!is_bilan(AccountType::Expense));
    }

    /// Un compte à nature créditrice est re-signé : un passif de 70 (crédit) vaut
    /// −70 en sens débit. Sans ce re-signe, le contrôle d'ouverture de la balance
    /// rougirait sur des livres exacts.
    #[test]
    fn le_sens_debit_re_signe_les_comptes_a_nature_creditrice() {
        assert_eq!(debit_sense(AccountType::Asset, d(70)), d(70));
        assert_eq!(debit_sense(AccountType::Expense, d(70)), d(70));
        assert_eq!(debit_sense(AccountType::Liability, d(70)), d(-70));
        assert_eq!(debit_sense(AccountType::Revenue, d(70)), d(-70));
    }

    /// Le bilan prend la paire « avant le début », le résultat la paire « depuis
    /// le début de l'exercice » — jamais l'inverse.
    #[test]
    fn l_ouverture_choisit_la_paire_selon_le_type() {
        let before = (d(1000), d(0));
        let since = (d(0), d(200));
        assert_eq!(opening_from(AccountType::Asset, before, since), d(1000));
        assert_eq!(opening_from(AccountType::Revenue, before, since), d(200));
        assert_eq!(
            opening_from(AccountType::Revenue, before, (d(0), d(0))),
            d(0)
        );
    }
}
