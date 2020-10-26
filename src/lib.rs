//! This crate makes it easier to parse tags/metadata in audio files of different file types.
//!
//! This crate aims to provide a unified trait for parsers and writers of different audio file formats. This means that you can parse tags in mp3 and m4a files with a single function: `Tag::default().read_from_path()` and get fields by directly calling `.album()`, `.artist()` on its result. Without this crate, you would otherwise need to learn different APIs in **id3**, **mp4ameta** crates in order to parse metadata in different file foramts.
//!
//! ## Example
//!
//! ```ignore
//! use audiotags::Tag;
//!
//! fn main() {
//!     const MP3: &'static str = "a.mp3";
//!     let mut tags = Tag::default().read_from_path(MP3).unwrap();
//!     // without this crate you would call id3::Tag::read_from_path()
//!     println!("Title: {:?}", tags.title());
//!     println!("Artist: {:?}", tags.artist());
//!     tags.set_album_artist("CINDERELLA PROJECT");
//!     let album = tags.album().unwrap();
//!     println!("Album title and artist: {:?}", (album.title, album.artist));
//!     println!("Track: {:?}", tags.track());
//!     tags.write_to_path(MP3).unwrap();
//! // Title: Some("お願い！シンデレラ")
//! // Artist: Some("高垣楓、城ヶ崎美嘉、小日向美穂、十時愛梨、川島瑞樹、日野茜、輿水幸子、佐久間まゆ、白坂小梅")
//! // Album title and artist: ("THE IDOLM@STER CINDERELLA GIRLS ANIMATION PROJECT 01 Star!!", Some("CINDERELLA PROJECT"))
//! // Track: (Some(2), Some(4))
//!
//!     const M4A: &'static str = "b.m4a";
//!     let mut tags = Tag::default().read_from_path(M4A).unwrap();
//!     // without this crate you would call mp4ameta::Tag::read_from_path()
//!     println!("Title: {:?}", tags.title());
//!     println!("Artist: {:?}", tags.artist());
//!     let album = tags.album().unwrap();
//!     println!("Album title and artist: {:?}", (album.title, album.artist));
//!     tags.set_total_tracks(4);
//!     println!("Track: {:?}", tags.track());
//!     tags.write_to_path(M4A).unwrap();
//! // Title: Some("ふわふわ時間")
//! // Artist: Some("桜高軽音部 [平沢唯・秋山澪・田井中律・琴吹紬(CV:豊崎愛生、日笠陽子、佐藤聡美、寿美菜子)]")
//! // Album title and artist: ("ふわふわ時間", Some("桜高軽音部 [平沢唯・秋山澪・田井中律・琴吹紬(CV:豊崎愛生、日笠陽子、佐藤聡美、寿美菜子)]"))
//! // Track: (Some(1), Some(4))
//! }
//! ```

mod id3_tag;
pub use id3_tag::Id3Tag;
mod flac_tag;
mod mp4_tag;
// use flac_tag::FlacTag;
use mp4_tag::Mp4Tag;

use std::convert::From;
use std::fs::File;
use std::path::Path;
use strum::Display;

use beef::lean::Cow;

use std::convert::{TryFrom, TryInto};

type BoxedError = Box<dyn std::error::Error>;

#[derive(Debug, Display)]
pub enum Error {
    UnsupportedFormat(String),
    UnsupportedMimeType(String),
    NotAPicture,
}

pub type StdResult<T, E> = std::result::Result<T, E>;
pub type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {}

#[derive(Clone, Copy, Debug)]
pub enum TagType {
    /// ## Common file extensions
    ///
    /// `.mp3`
    ///
    /// ## References
    ///
    /// - https://www.wikiwand.com/en/ID3
    Id3v2,
    Flac,
    /// ## Common file extensions
    ///
    /// `.mp4, .m4a, .m4p, .m4b, .m4r and .m4v`
    ///
    /// ## References
    ///
    /// - https://www.wikiwand.com/en/MPEG-4_Part_14
    Mp4,
}

#[rustfmt::skip]
impl TagType {
    fn try_from_ext(ext: &str) -> StdResult<Self, BoxedError> {
        match ext {
                                                     "mp3" => Ok(Self::Id3v2),
            "m4a" | "m4b" | "m4p" | "m4v" | "isom" | "mp4" => Ok(Self::Mp4),
                                                    "flac" => Ok(Self::Flac),
            p @ _ => Err(Box::new(Error::UnsupportedFormat(p.to_owned()))),
        }
    }
}

#[derive(Default)]
pub struct Tag {
    tag_type: Option<TagType>,
}

// impl Tag {
//     pub fn with_tag_type(tag_type: TagType) -> Self {
//         Self {
//             tag_type: Some(tag_type),
//         }
//     }

//     pub fn read_from_path(
//         &self,
//         path: impl AsRef<Path>,
//     ) -> Result<Box<dyn AudioTagsIo>, BoxedError> {
//         match self.tag_type.unwrap_or(TagType::try_from_ext(
//             path.as_ref()
//                 .extension()
//                 .unwrap()
//                 .to_string_lossy()
//                 .to_string()
//                 .to_lowercase()
//                 .as_str(),
//         )?) {
//             TagType::Id3v2 => Ok(Box::new(Id3Tag::read_from_path(path)?)),
//             TagType::Mp4 => Ok(Box::new(Mp4Tag::read_from_path(path)?)),
//             TagType::Flac => Ok(Box::new(FlacTag::read_from_path(path)?)),
//         }
//     }
// }

// // ? deprecate?
// /// Guesses the audio metadata handler from the file extension, and returns the `Box`ed IO handler.
// pub fn read_from_path(path: impl AsRef<Path>) -> Result<Box<dyn AudioTagsIo>, BoxedError> {
//     Tag::default().read_from_path(path)
// }

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MimeType {
    Png,
    Jpeg,
    Tiff,
    Bmp,
    Gif,
}

impl TryFrom<&str> for MimeType {
    type Error = Error;
    fn try_from(inp: &str) -> Result<Self> {
        Ok(match inp {
            "image/jpeg" => MimeType::Jpeg,
            "image/png" => MimeType::Png,
            "image/tiff" => MimeType::Tiff,
            "image/bmp" => MimeType::Bmp,
            "image/gif" => MimeType::Gif,
            _ => return Err(Error::UnsupportedMimeType(inp.to_owned())),
        })
    }
}

impl From<MimeType> for String {
    fn from(mt: MimeType) -> Self {
        match mt {
            MimeType::Jpeg => "image/jpeg".to_owned(),
            MimeType::Png => "image/png".to_owned(),
            MimeType::Tiff => "image/tiff".to_owned(),
            MimeType::Bmp => "image/bmp".to_owned(),
            MimeType::Gif => "image/gif".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Picture<'a> {
    pub data: Cow<'a, [u8]>,
    pub mime_type: MimeType,
}

impl<'a> Picture<'a> {
    pub fn try_with_mime(data: Vec<u8>, mime: &str) -> Result<Self> {
        let mime_type: MimeType = mime.try_into()?;
        Ok(Self {
            data: Cow::owned(data),
            mime_type,
        })
    }
}

/// A struct for representing an album for convinience.
#[derive(Debug)]
pub struct Album<'a> {
    pub title: Cow<'a, str>,
    pub artist: Option<Cow<'a, str>>,
    pub cover: Option<Picture<'a>>,
}

impl<'a> Album<'a> {
    pub fn with_title(title: impl Into<String>) -> Self {
        Self {
            title: Cow::owned(title.into()),
            artist: None,
            cover: None,
        }
    }
    pub fn and_artist(mut self, artist: impl Into<String>) -> Self {
        self.artist = Some(Cow::owned(artist.into()));
        self
    }
    pub fn and_cover(mut self, cover: Picture<'a>) -> Self {
        self.cover = Some(cover);
        self
    }
    pub fn with_all(
        title: impl Into<String>,
        artist: impl Into<String>,
        cover: Picture<'a>,
    ) -> Self {
        Self {
            title: Cow::owned(title.into()),
            artist: Some(Cow::owned(artist.into())),
            cover: Some(cover),
        }
    }
}

const SEP_ARTIST: &'static str = ";";

#[derive(Default)]
pub struct AnyTag<'a> {
    pub title: Option<Cow<'a, str>>,
    // pub artists: Option<Vec<Cow<'a, str>>>,  // ? iterator
    pub year: Option<i32>,
    pub album_title: Option<Cow<'a, str>>,
    // pub album_artists: Option<Vec<Cow<'a, str>>>, // ? iterator
    pub album_cover: Option<Picture<'a>>,
    pub track_number: Option<u16>,
    pub total_tracks: Option<u16>,
    pub disc_number: Option<u16>,
    pub total_discs: Option<u16>,
}

impl<'a> AnyTag<'a> {
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }
    // pub fn artists(&self) -> Option<&[String]> {
    //     self.artists.as_deref()
    // }
    pub fn year(&self) -> Option<i32> {
        self.year
    }
    pub fn album_title(&self) -> Option<&str> {
        self.album_title.as_deref()
    }
    // pub fn album_artists(&self) -> Option<&[String]> {
    //     self.album_artists.as_deref()
    // }
    pub fn track_number(&self) -> Option<u16> {
        self.track_number
    }
    pub fn total_tracks(&self) -> Option<u16> {
        self.total_tracks
    }
    pub fn disc_number(&self) -> Option<u16> {
        self.track_number
    }
    pub fn total_discs(&self) -> Option<u16> {
        self.total_tracks
    }
}

pub trait TagIo {
    fn read_from_path(path: &str) -> StdResult<AnyTag, BoxedError>;
    fn write_to_path(path: &str) -> StdResult<(), BoxedError>;
}

// impl<'a> AnyTag<'a> {
//     fn read_from_path<>
// }

fn read_from_path<T>(path: &str) -> StdResult<AnyTag, BoxedError>
where
    T: TagIo,
{
    T::read_from_path(path)
}

// Implementors of this trait are able to read and write audio metadata.
//
// Constructor methods e.g. `from_file` should be implemented separately.
// pub trait AudioTagsIo {
//     fn title(&self) -> Option<Cow<str>>;
//     fn set_title(&mut self, title: &str);
//     fn remove_title(&mut self);

//     fn artist(&self) -> Option<&str>;
//     fn set_artist(&mut self, artist: &str);
//     fn remove_artist(&mut self);

//     fn year(&self) -> Option<i32>;
//     fn set_year(&mut self, year: i32);
//     fn remove_year(&mut self);

//     fn album(&self) -> Option<Album> {
//         self.album_title().map(|title| Album {
//             title: title.to_owned(),
//             artist: self.album_artist().map(|x| x.to_owned()),
//             cover: self.album_cover(),
//         })
//     }
//     fn set_album(&mut self, album: Album) {
//         self.set_album_title(&album.title);
//         if let Some(artist) = album.artist {
//             self.set_album_artist(&artist)
//         } else {
//             self.remove_album_artist()
//         }
//         if let Some(pic) = album.cover {
//             self.set_album_cover(pic)
//         } else {
//             self.remove_album_cover()
//         }
//     }
//     fn remove_album(&mut self) {
//         self.remove_album_title();
//         self.remove_album_artist();
//         self.remove_album_cover();
//     }

//     fn album_title(&self) -> Option<&str>;
//     fn set_album_title(&mut self, v: &str);
//     fn remove_album_title(&mut self);

//     fn album_artist(&self) -> Option<&str>;
//     fn set_album_artist(&mut self, v: &str);
//     fn remove_album_artist(&mut self);

//     fn album_cover(&self) -> Option<Picture>;
//     fn set_album_cover(&mut self, cover: Picture);
//     fn remove_album_cover(&mut self);

//     fn track(&self) -> (Option<u16>, Option<u16>) {
//         (self.track_number(), self.total_tracks())
//     }
//     fn set_track(&mut self, track: (u16, u16)) {
//         self.set_track_number(track.0);
//         self.set_total_tracks(track.1);
//     }
//     fn remove_track(&mut self) {
//         self.remove_track_number();
//         self.remove_total_tracks();
//     }

//     fn track_number(&self) -> Option<u16>;
//     fn set_track_number(&mut self, track_number: u16);
//     fn remove_track_number(&mut self);

//     fn total_tracks(&self) -> Option<u16>;
//     fn set_total_tracks(&mut self, total_track: u16);
//     fn remove_total_tracks(&mut self);

//     fn disc(&self) -> (Option<u16>, Option<u16>) {
//         (self.disc_number(), self.total_discs())
//     }
//     fn set_disc(&mut self, disc: (u16, u16)) {
//         self.set_disc_number(disc.0);
//         self.set_total_discs(disc.1);
//     }
//     fn remove_disc(&mut self) {
//         self.remove_disc_number();
//         self.remove_total_discs();
//     }

//     fn disc_number(&self) -> Option<u16>;
//     fn set_disc_number(&mut self, disc_number: u16);
//     fn remove_disc_number(&mut self);

//     fn total_discs(&self) -> Option<u16>;
//     fn set_total_discs(&mut self, total_discs: u16);
//     fn remove_total_discs(&mut self);

//     fn write_to(&mut self, file: &mut File) -> Result<(), BoxedError>;
//     // cannot use impl AsRef<Path>
//     fn write_to_path(&mut self, path: &str) -> Result<(), BoxedError>;
// }

// impl AnyTag {
//     pub fn artists_as_string(&self, sep: &str) -> Option<String> {
//         self.artists().map(|artists| artists.join(sep))
//     }
//     pub fn album_artists_as_string(&self, sep: &str) -> Option<String> {
//         self.album_artists().map(|artists| artists.join(sep))
//     }
// }

// #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
// pub enum PictureType {
//     Other,
//     Icon,
//     OtherIcon,
//     CoverFront,
//     CoverBack,
//     Leaflet,
//     Media,
//     LeadArtist,
//     Artist,
//     Conductor,
//     Band,
//     Composer,
//     Lyricist,
//     RecordingLocation,
//     DuringRecording,
//     DuringPerformance,
//     ScreenCapture,
//     BrightFish,
//     Illustration,
//     BandLogo,
//     PublisherLogo,
//     Undefined(u8),
// }
