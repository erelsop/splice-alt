use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleMetadata {
    pub sample: Sample,
    pub sample_meta_data: SampleMetaData,
    pub remaining_credits: Option<u32>,
    pub purchase_etag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    pub url: String,
    pub path: String,
    pub sas_id: String,
    pub file_hash: String,
    pub file_size: u64,
    pub encoding: Encoding,
    #[serde(rename = "type")]
    pub sample_type: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encoding {
    pub name: String,
    pub decoded_format: String,
    pub decoded_hash: String,
    pub decoded_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SampleMetaData {
    pub audio_key: Option<String>,
    pub bpm: Option<u32>,
    pub chord_type: Option<String>,
    pub dir: String,
    pub duration: u32,
    pub file_hash: String,
    pub filename: String,
    pub pack: Pack,
    pub preview_url: String,
    pub price: u32,
    pub provider_name: String,
    pub provider_uuid: String,
    pub provider_permalink: String,
    pub sample_type: String,
    pub tags: Vec<String>,
    pub waveform_url: String,
    pub published: bool,
    pub popularity: u32,
    pub trending: u32,
    pub published_at: String,
    pub purchased_at: String,
    pub sas_id: String,
    pub liked: bool,
    pub licensed: bool,
    pub asset_uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pack {
    pub uuid: String,
    pub name: String,
    pub description: String,
    pub provider_name: String,
    pub provider_description: String,
    pub cover_url: String,
    pub banner_url: String,
    pub main_genre: String,
    pub sample_count: u32,
    pub preset_count: u32,
    pub permalink: String,
    pub is_archived: bool,
}

/// Bitwig Studio compatible categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BitwigCategory {
    Bass,
    Bell,
    Brass,
    Chip,
    Cymbal,
    Drone,
    DrumLoop,
    Guitar,
    HiHat,
    Keyboards,
    Kick,
    Lead,
    Mallet,
    Orchestral,
    Organ,
    OtherDrums,
    Pad,
    Percussion,
    Piano,
    Snare,
    SoundFX,
    Strings,
    Synth,
    Tom,
    Unknown,
    Vocal,
    Winds,
}

impl BitwigCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            BitwigCategory::Bass => "Bass",
            BitwigCategory::Bell => "Bell",
            BitwigCategory::Brass => "Brass",
            BitwigCategory::Chip => "Chip",
            BitwigCategory::Cymbal => "Cymbal",
            BitwigCategory::Drone => "Drone",
            BitwigCategory::DrumLoop => "Drum Loop",
            BitwigCategory::Guitar => "Guitar",
            BitwigCategory::HiHat => "Hi-hat",
            BitwigCategory::Keyboards => "Keyboards",
            BitwigCategory::Kick => "Kick",
            BitwigCategory::Lead => "Lead",
            BitwigCategory::Mallet => "Mallet",
            BitwigCategory::Orchestral => "Orchestral",
            BitwigCategory::Organ => "Organ",
            BitwigCategory::OtherDrums => "Other Drums",
            BitwigCategory::Pad => "Pad",
            BitwigCategory::Percussion => "Percussion",
            BitwigCategory::Piano => "Piano",
            BitwigCategory::Snare => "Snare",
            BitwigCategory::SoundFX => "Sound FX",
            BitwigCategory::Strings => "Strings",
            BitwigCategory::Synth => "Synth",
            BitwigCategory::Tom => "Tom",
            BitwigCategory::Unknown => "Unknown",
            BitwigCategory::Vocal => "Vocal",
            BitwigCategory::Winds => "Winds",
        }
    }
}

impl FromStr for BitwigCategory {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bass" => Ok(BitwigCategory::Bass),
            "bell" => Ok(BitwigCategory::Bell),
            "brass" => Ok(BitwigCategory::Brass),
            "chip" => Ok(BitwigCategory::Chip),
            "cymbal" => Ok(BitwigCategory::Cymbal),
            "drone" => Ok(BitwigCategory::Drone),
            "drum loop" | "drumloop" => Ok(BitwigCategory::DrumLoop),
            "guitar" => Ok(BitwigCategory::Guitar),
            "hi-hat" | "hihat" => Ok(BitwigCategory::HiHat),
            "keyboards" => Ok(BitwigCategory::Keyboards),
            "kick" => Ok(BitwigCategory::Kick),
            "lead" => Ok(BitwigCategory::Lead),
            "mallet" => Ok(BitwigCategory::Mallet),
            "orchestral" => Ok(BitwigCategory::Orchestral),
            "organ" => Ok(BitwigCategory::Organ),
            "other drums" | "otherdrums" => Ok(BitwigCategory::OtherDrums),
            "pad" => Ok(BitwigCategory::Pad),
            "percussion" => Ok(BitwigCategory::Percussion),
            "piano" => Ok(BitwigCategory::Piano),
            "snare" => Ok(BitwigCategory::Snare),
            "sound fx" | "soundfx" | "fx" => Ok(BitwigCategory::SoundFX),
            "strings" => Ok(BitwigCategory::Strings),
            "synth" => Ok(BitwigCategory::Synth),
            "tom" => Ok(BitwigCategory::Tom),
            "unknown" => Ok(BitwigCategory::Unknown),
            "vocal" => Ok(BitwigCategory::Vocal),
            "winds" => Ok(BitwigCategory::Winds),
            _ => Err(format!("Invalid category: {}", s)),
        }
    }
}

/// Maps Splice tags to Bitwig categories
pub fn map_tags_to_category(tags: &[String]) -> BitwigCategory {
    let tags_lower: Vec<String> = tags.iter().map(|t| t.to_lowercase()).collect();
    
    // Define mapping rules based on common Splice tags
    for tag in &tags_lower {
        match tag.as_str() {
            // Drum elements
            "kick" | "kicks" => return BitwigCategory::Kick,
            "snare" | "snares" => return BitwigCategory::Snare,
            "hihat" | "hi-hat" | "hihats" | "hi-hats" => return BitwigCategory::HiHat,
            "cymbal" | "cymbals" => return BitwigCategory::Cymbal,
            "tom" | "toms" => return BitwigCategory::Tom,
            "percussion" | "perc" => return BitwigCategory::Percussion,
            "drum loop" | "drum loops" | "drums" => return BitwigCategory::DrumLoop,
            
            // Melodic elements
            "bass" | "bassline" | "sub bass" => return BitwigCategory::Bass,
            "lead" | "leads" | "lead synth" => return BitwigCategory::Lead,
            "pad" | "pads" | "ambient" => return BitwigCategory::Pad,
            "synth" | "synthesizer" => return BitwigCategory::Synth,
            
            // Instruments
            "piano" => return BitwigCategory::Piano,
            "guitar" => return BitwigCategory::Guitar,
            "organ" => return BitwigCategory::Organ,
            "bell" | "bells" => return BitwigCategory::Bell,
            "brass" => return BitwigCategory::Brass,
            "strings" | "string" => return BitwigCategory::Strings,
            "vocal" | "vocals" | "voice" => return BitwigCategory::Vocal,
            
            // Effects
            "fx" | "sfx" | "sound fx" | "effects" => return BitwigCategory::SoundFX,
            "drone" | "texture" => return BitwigCategory::Drone,
            
            _ => continue,
        }
    }
    
    // Default to Unknown if no mapping found
    BitwigCategory::Unknown
}

impl SampleMetadata {
    /// Load metadata from a JSON file
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let metadata: SampleMetadata = serde_json::from_str(&content)?;
        Ok(metadata)
    }
    
    /// Get the mapped Bitwig category for this sample
    pub fn get_category(&self) -> BitwigCategory {
        map_tags_to_category(&self.sample_meta_data.tags)
    }
    
    /// Generate the target library path for this sample
    pub fn get_library_path(&self, library_base: &std::path::Path) -> std::path::PathBuf {
        let category = self.get_category();
        let pack_name = &self.sample_meta_data.pack.name;
        let filename = &self.sample_meta_data.filename;
        
        // Sanitize pack name for filesystem
        let safe_pack_name = sanitize_filename(pack_name);
        
        library_base
            .join(category.as_str())
            .join(safe_pack_name)
            .join(filename)
    }
}

/// Sanitize a filename by replacing problematic characters with safe alternatives
pub fn sanitize_filename(name: &str) -> String {
    // Replace problematic characters with safe alternatives
    name.chars()
        .filter_map(|c| match c {
            '/' | '\\' => Some('-'),
            ':' => Some('-'),
            '*' | '?' | '\0' => None, // Remove these characters entirely
            '"' => Some('\''),
            '<' | '>' => Some('-'),
            '|' => Some('-'),
            c if c.is_control() => Some('_'),
            c => Some(c),
        })
        .collect::<String>()
        .trim()
        .to_string()
} 