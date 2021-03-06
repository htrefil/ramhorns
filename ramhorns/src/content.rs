// Ramhorns  Copyright (C) 2019  Maciej Hirsz
//
// This file is part of Ramhorns. This program comes with ABSOLUTELY NO WARRANTY;
// This is free software, and you are welcome to redistribute it under the
// conditions of the GNU General Public License version 3.0.
//
// You should have received a copy of the GNU General Public License
// along with Ramhorns.  If not, see <http://www.gnu.org/licenses/>

use crate::{Template, Section};
use crate::encoding::Encoder;

use std::borrow::Borrow;
use std::collections::{HashMap, BTreeMap};
use std::hash::Hash;

/// Trait allowing the rendering to quickly access data stored in the type that
/// implements it. You needn't worry about implementing it, in virtually all
/// cases the `#[derive(Content)]` attribute above your types should be sufficient.
pub trait Content: Sized {
    /// Marks whether this content is truthy. Used when attempting to render a section.
    fn is_truthy(&self) -> bool {
        true
    }

    /// How much capacity is _likely_ required for all the data in this `Content`
    /// for a given `Template`.
    fn capacity_hint(&self, _tpl: &Template) -> usize {
        0
    }

    /// Renders self as a variable to the encoder.
    ///
    /// This will escape HTML characters, eg: `<` will become `&lt;`.
    fn render_escaped<'section, E>(&self, _encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        Ok(())
    }

    /// Renders self as a variable to the encoder.
    ///
    /// This doesn't perform any escaping at all.
    fn render_unescaped<'section, E>(&self, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        self.render_escaped(encoder)
    }

    /// Renders self as a variable to the encoder with CommonMark processing.
    ///
    /// The generated HTML is never escaped.
    fn render_cmark<'section, E>(&self, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        self.render_escaped(encoder)
    }

    /// Render a section with self.
    fn render_section<'section, E>(&self, section: Section<'section>, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        if self.is_truthy() {
            section.render_once(self, encoder)
        } else {
            Ok(())
        }
    }

    /// Render a section with self.
    fn render_inverse<'section, E>(&self, section: Section<'section>, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        if !self.is_truthy() {
            section.render_once(self, encoder)
        } else {
            Ok(())
        }
    }

    /// Render a field by the hash **or** string of its name.
    ///
    /// This will escape HTML characters, eg: `<` will become `&lt;`.
    fn render_field_escaped<E>(&self, _hash: u64, _name: &str, _encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        Ok(())
    }

    /// Render a field by the hash **or** string of its name.
    ///
    /// This doesn't perform any escaping at all.
    fn render_field_unescaped<E>(&self, _hash: u64, _name: &str, _encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        Ok(())
    }

    /// Render a field by the hash **or** string of its name, as a section.
    fn render_field_section<'section, E>(&self, _hash: u64, _name: &str, _section: Section<'section>, _encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        Ok(())
    }

    /// Render a field, by the hash of **or** string its name, as an inverse section.
    fn render_field_inverse<'section, E>(&self, _hash: u64, _name: &str, _section: Section<'section>, _encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        Ok(())
    }
}

impl Content for &str {
    fn is_truthy(&self) -> bool {
        self.len() != 0
    }

    fn capacity_hint(&self, _tpl: &Template) -> usize {
        self.len()
    }

    fn render_escaped<'section, E>(&self, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.write_escaped(*self)
    }

    fn render_unescaped<'section, E>(&self, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.write_unescaped(*self)
    }

    fn render_cmark<'section, E>(&self, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        crate::cmark::encode(*self, encoder)
    }
}

impl Content for String {
    fn is_truthy(&self) -> bool {
        self.len() != 0
    }

    fn capacity_hint(&self, _tpl: &Template) -> usize {
        self.len()
    }

    fn render_escaped<'section, E>(&self, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.write_escaped(self)
    }

    fn render_unescaped<'section, E>(&self, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        encoder.write_unescaped(self)
    }

    fn render_cmark<'section, E>(&self, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        crate::cmark::encode(self, encoder)
    }
}

impl Content for bool {
    fn is_truthy(&self) -> bool {
        *self
    }

    fn capacity_hint(&self, _tpl: &Template) -> usize {
        5
    }

    fn render_escaped<'section, E>(&self, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        // Nothing to escape here
        encoder.write_unescaped(match *self {
            true => "true",
            false => "false",
        })
    }
}

macro_rules! impl_number_types {
    ($( $ty:ty ),*) => {
        $(
            impl Content for $ty {
                fn is_truthy(&self) -> bool {
                    *self != 0 as $ty
                }

                fn capacity_hint(&self, _tpl: &Template) -> usize {
                    5
                }

                fn render_escaped<'section, E>(&self, encoder: &mut E) -> Result<(), E::Error>
                where
                    E: Encoder,
                {
                    // Nothing to escape here
                    encoder.format_unescaped(self)
                }
            }
        )*
    }
}

impl_number_types!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);

impl<T: Content> Content for Option<T> {
    fn is_truthy(&self) -> bool {
        self.is_some()
    }

    fn capacity_hint(&self, tpl: &Template) -> usize {
        match self {
            Some(inner) => inner.capacity_hint(tpl),
            _ => 0
        }
    }

    fn render_escaped<'section, E>(&self, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        if let Some(inner) = self {
            inner.render_escaped(encoder)?;
        }

        Ok(())
    }

    fn render_unescaped<'section, E>(&self, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        if let Some(ref inner) = self {
            inner.render_unescaped(encoder)?;
        }

        Ok(())
    }

    fn render_section<'section, E>(&self, section: Section<'section>, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        if let Some(item) = self {
            section.render_once(item, encoder)?;
        }

        Ok(())
    }
}

impl<T: Content, U> Content for Result<T, U> {
    fn is_truthy(&self) -> bool {
        self.is_ok()
    }

    fn capacity_hint(&self, tpl: &Template) -> usize {
        match self {
            Ok(inner) => inner.capacity_hint(tpl),
            _ => 0
        }
    }

    fn render_escaped<'section, E>(&self, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        if let Ok(inner) = self {
            inner.render_escaped(encoder)?;
        }

        Ok(())
    }

    fn render_unescaped<'section, E>(&self, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        if let Ok(ref inner) = self {
            inner.render_unescaped(encoder)?;
        }

        Ok(())
    }

    fn render_section<'section, E>(&self, section: Section<'section>, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        if let Ok(item) = self {
            section.render_once(item, encoder)?;
        }

        Ok(())
    }
}

impl<T: Content> Content for Vec<T> {
    fn is_truthy(&self) -> bool {
        self.len() != 0
    }

    fn render_section<'section, E>(&self, section: Section<'section>, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        for item in self.iter() {
            section.render_once(item, encoder)?;
        }

        Ok(())
    }
}

impl<T: Content> Content for &[T] {
    fn is_truthy(&self) -> bool {
        self.len() != 0
    }

    fn render_section<'section, E>(&self, section: Section<'section>, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        for item in self.iter() {
            section.render_once(item, encoder)?;
        }

        Ok(())
    }
}

impl<K, V> Content for HashMap<K, V>
where
    K: Borrow<str> + Hash + Eq,
    V: Content,
{
    fn is_truthy(&self) -> bool {
        !self.is_empty()
    }

    fn render_field_escaped<E>(&self, _: u64, name: &str, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        match self.get(name) {
            Some(v) => v.render_escaped(encoder),
            None => Ok(())
        }
    }

    fn render_field_unescaped<E>(&self, _: u64, name: &str, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        match self.get(name) {
            Some(v) => v.render_unescaped(encoder),
            None => Ok(())
        }
    }

    fn render_field_section<'section, E>(&self, _: u64, name: &str, section: Section<'section>, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        match self.get(name) {
            Some(v) => v.render_section(section, encoder),
            None => Ok(())
        }
    }

    fn render_field_inverse<'section, E>(&self, _: u64, name: &str, section: Section<'section>, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        match self.get(name) {
            Some(v) => v.render_inverse(section, encoder),
            None => Ok(())
        }
    }
}

impl<K, V> Content for BTreeMap<K, V>
where
    K: Borrow<str> + Ord,
    V: Content,
{
    fn is_truthy(&self) -> bool {
        !self.is_empty()
    }

    fn render_field_escaped<E>(&self, _: u64, name: &str, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        match self.get(name) {
            Some(v) => v.render_escaped(encoder),
            None => Ok(())
        }
    }

    fn render_field_unescaped<E>(&self, _: u64, name: &str, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        match self.get(name) {
            Some(v) => v.render_unescaped(encoder),
            None => Ok(())
        }
    }

    fn render_field_section<'section, E>(&self, _: u64, name: &str, section: Section<'section>, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        match self.get(name) {
            Some(v) => v.render_section(section, encoder),
            None => Ok(())
        }
    }

    fn render_field_inverse<'section, E>(&self, _: u64, name: &str, section: Section<'section>, encoder: &mut E) -> Result<(), E::Error>
    where
        E: Encoder,
    {
        match self.get(name) {
            Some(v) => v.render_inverse(section, encoder),
            None => Ok(())
        }
    }
}
