use beancount::core::*;
use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use thiserror::Error;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Debug)]
pub struct BasicRenderer {}

impl BasicRenderer {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn render<W: Write>(w: &mut W, document: &Document<'_>) -> Result<(), BasicRendererError>{
    BasicRenderer::default().render(document, w)
}

#[derive(Error, Debug)]
pub enum BasicRendererError {
    #[error("an io error occurred")]
    Io(#[from] io::Error),
    #[error("could not render unsupported directive")]
    Unsupported,
}

pub trait Renderer<T, W: Write> {
    type Error;
    fn render(&self, renderable: T, write: &mut W) -> Result<(), Self::Error>;
}

impl<'a, W: Write> Renderer<&'a Ledger<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, ledger: &'a Ledger<'_>, write: &mut W) -> Result<(), Self::Error> {
        for directive in &ledger.directives {
            self.render(directive, write)?;
            writeln!(write, "")?;
        }
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a Document<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, document: &'a Document<'_>, write: &mut W) -> Result<(), Self::Error> {
        // TODO: Tags? Links?
        write!(write, "{} document ", document.date)?;
        self.render(&document.account, write)?;
        writeln!(write, " \"{}\"", document.path)?;
        render_key_value(write, &document.meta)?;
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a Directive<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, directive: &'a Directive<'_>, write: &mut W) -> Result<(), Self::Error> {
        use Directive::*;
        match directive {
            Open(open) => self.render(open, write),
            Close(close) => self.render(close, write),
            Balance(balance) => self.render(balance, write),
            Option(bc_option) => self.render(bc_option, write),
            Commodity(commodity) => self.render(commodity, write),
            Custom(custom) => self.render(custom, write),
            Document(document) => self.render(document, write),
            Event(event) => self.render(event, write),
            Include(include) => self.render(include, write),
            Note(note) => self.render(note, write),
            Pad(pad) => self.render(pad, write),
            Plugin(plugin) => self.render(plugin, write),
            Price(price) => self.render(price, write),
            Query(query) => self.render(query, write),
            Transaction(transaction) => self.render(transaction, write),
            Unsupported => return Err(BasicRendererError::Unsupported),
        }
    }
}

fn render_key_value<W: Write>(
    w: &mut W,
    kv: &HashMap<&str, &str>,
) -> Result<(), BasicRendererError> {
    for (key, value) in kv {
        writeln!(w, "\t{}: {}", key, value)?;
    }
    Ok(())
}

impl<'a, W: Write> Renderer<&'a Open<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, open: &'a Open<'_>, write: &mut W) -> Result<(), Self::Error> {
        write!(write, "{} open ", open.date)?;
        self.render(&open.account, write)?;
        for (i, currency) in open.currencies.iter().enumerate() {
            write!(write, "{}", currency)?;
            if i < open.currencies.len() - 1 {
                write!(write, " ")?;
            }
        }
        match open.booking {
            Booking::Strict => write!(write, r#" "strict""#)?,
            Booking::None => {}
            Booking::Average => write!(write, r#" "average""#)?,
            Booking::Fifo => write!(write, r#" "fifo""#)?,
            Booking::Lifo => write!(write, r#" "lifo""#)?,
        };
        writeln!(write, "")?;
        render_key_value(write, &open.meta)?;
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a Close<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, close: &'a Close<'_>, write: &mut W) -> Result<(), Self::Error> {
        write!(write, "{} close ", close.date)?;
        self.render(&close.account, write)?;
        writeln!(write, "")?;
        render_key_value(write, &close.meta)?;
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a Account<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, account: &'a Account<'_>, write: &mut W) -> Result<(), Self::Error> {
        write!(
            write,
            "{}:{}",
            match account.ty {
                AccountType::Assets => "Assets",
                AccountType::Liabilities => "Liabilities",
                AccountType::Equity => "Equity",
                AccountType::Income => "Income",
                AccountType::Expenses => "Expenses",
            },
            account.parts.join(":")
        )?;
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a Balance<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, balance: &'a Balance<'_>, w: &mut W) -> Result<(), Self::Error> {
        write!(w, "{} balance ", balance.date)?;
        self.render(&balance.account, w)?;
        write!(w, "\t")?;
        self.render(&balance.amount, w)?;
        writeln!(w, "")?;
        render_key_value(w, &balance.meta)?;
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a Amount<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, amount: &'a Amount<'_>, w: &mut W) -> Result<(), Self::Error> {
        write!(w, "{} {}", amount.num, amount.currency)?;
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a BcOption<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, option: &'a BcOption<'_>, w: &mut W) -> Result<(), Self::Error> {
        writeln!(w, "option \"{}\" \"{}\"", option.name, option.val)?;
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a Commodity<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, commodity: &'a Commodity<'_>, w: &mut W) -> Result<(), Self::Error> {
        writeln!(w, "{} commodity {}", commodity.date, commodity.name)?;
        render_key_value(w, &commodity.meta)
    }
}

impl<'a, W: Write> Renderer<&'a Custom<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, custom: &'a Custom<'_>, w: &mut W) -> Result<(), Self::Error> {
        writeln!(
            w,
            "{} custom \"{}\" {}",
            custom.date,
            custom.name,
            custom.args.join(" ")
        )?;
        render_key_value(w, &custom.meta)
    }
}

impl<'a, W: Write> Renderer<&'a Event<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, event: &'a Event<'_>, w: &mut W) -> Result<(), Self::Error> {
        writeln!(w, "{} event \"{}\" \"{}\"", event.date, event.name, event.description)?;
        render_key_value(w, &event.meta)
    }
}

impl<'a, W: Write> Renderer<&'a Include<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, include: &'a Include<'_>, w: &mut W) -> Result<(), Self::Error> {
        writeln!(w, "include {}", include.filename)?;
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a Note<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, note: &'a Note<'_>, w: &mut W) -> Result<(), Self::Error> {
        write!(w, "{} note ", note.date)?;
        self.render(&note.account, w)?;
        writeln!(w, " \"{}\"", note.comment)?;
        render_key_value(w, &note.meta)
    }
}

impl<'a, W: Write> Renderer<&'a Pad<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, pad: &'a Pad<'_>, w: &mut W) -> Result<(), Self::Error> {
        write!(w, "{} pad ", pad.date)?;
        self.render(&pad.pad_to_account, w)?;
        write!(w, " ")?;
        self.render(&pad.pad_from_account, w)?;
        writeln!(w, "")?;
        render_key_value(w, &pad.meta)
    }
}

impl<'a, W: Write> Renderer<&'a Plugin<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, plugin: &'a Plugin<'_>, w: &mut W) -> Result<(), Self::Error> {
        write!(w, "plugin \"{}\"", plugin.module)?;
        if let Some(config) = &plugin.config {
            write!(w, " \"{}\"", config)?;
        }
        writeln!(w, "")?;
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a Price<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, price: &'a Price<'_>, w: &mut W) -> Result<(), Self::Error> {
        write!(w, "{} price {} ", price.date, price.currency)?;
        self.render(&price.amount, w)?;
        writeln!(w, "")?;
        render_key_value(w, &price.meta)
    }
}

impl<'a, W: Write> Renderer<&'a Query<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, query: &'a Query<'_>, w: &mut W) -> Result<(), Self::Error> {
        writeln!(w, "{} query \"{}\" \"{}\"", query.date, query.name, query.query_string)?;
        render_key_value(w, &query.meta)
    }
}

impl<'a, W: Write> Renderer<&'a Transaction<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, transaction: &'a Transaction<'_>, w: &mut W) -> Result<(), Self::Error> {
        write!(w, "{} ", transaction.date)?;
        self.render(&transaction.flag, w)?;
        if let Some(payee) = &transaction.payee {
            write!(w, " \"{}\"", payee)?;
        }
        write!(w, " \"{}\"", &transaction.narration)?;
        for tag in &transaction.tags {
            write!(w, " {}", tag)?;
        }
        for link in &transaction.links {
            write!(w, " {}", link)?;
        }
        for posting in &transaction.postings {
            self.render(posting, w)?;
        }
        render_key_value(w, &transaction.meta)
    }
}

impl<'a, W: Write> Renderer<&'a Posting<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, posting: &'a Posting<'_>, w: &mut W) -> Result<(), Self::Error> {
        write!(w, "\t")?;
        if let Some(flag) = &posting.flag {
            self.render(flag, w)?;
            write!(w, " ")?;
        }
        self.render(&posting.account, w)?;
        write!(w, "\t")?;
        self.render(&posting.units, w)?;
        if let Some(price) = &posting.price {
            write!(w, " @ ")?;
            self.render(price, w)?;
        }
        if let Some(cost) = &posting.cost {
            write!(w, " ")?;
            self.render(cost, w)?;
        }
        render_key_value(w, &posting.meta)
    }
}

impl<'a, W: Write> Renderer<&'a Flag, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, flag: &'a Flag, w: &mut W) -> Result<(), Self::Error> {
        match flag {
            Flag::Okay => write!(w, "*")?,
            Flag::Warning => write!(w, "!")?,
            Flag::Other(s) => write!(w, "{}", s)?
        };
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a CostSpec<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, cost: &'a CostSpec<'_>, w: &mut W) -> Result<(), Self::Error> {
        let double_brackets = cost.number_total.is_some();
        if double_brackets {
            write!(w, "{{{{")?;
        } else {
            write!(w, "{{")?;
        }
        let mut first = true;

        if let (Some(cost), Some(currency)) = (&cost.number_total.or(cost.number_per), &cost.currency) {
            write!(w, "{} {}", cost, currency)?;
            first = false;
        }

        if let Some(date) = &cost.date {
            if !first {
                write!(w, ", ")?;
            }
            write!(w, "{}", date)?;
            first = false;
        }

        if let Some(label) = &cost.label {
            if !first {
                write!(w, ", ")?;
            }
            write!(w, "{}", label)?;
        }

        if double_brackets {
            write!(w, "}}}}")?;
        } else {
            write!(w, "}}")?;
        }
        Ok(())
    }
}

impl<'a, W: Write> Renderer<&'a IncompleteAmount<'_>, W> for BasicRenderer {
    type Error = BasicRendererError;
    fn render(&self, incomplete_amount: &'a IncompleteAmount<'_>, w: &mut W) -> Result<(), Self::Error> {
        match (&incomplete_amount.num, &incomplete_amount.currency) {
            (Some(num), Some(currency)) => write!(w, "{} {}", num, currency),
            (None, Some(currency)) => write!(w, "{}", currency),
            (Some(num), None) => write!(w, "{}", num),
            _ => write!(w, ""),
        }?;
        Ok(())
    }
}
