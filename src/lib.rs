use beancount::core::*;
use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use thiserror::Error;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Default, Debug)]
pub struct BasicRenderer {}

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
            write!(write, "{}", currency);
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
