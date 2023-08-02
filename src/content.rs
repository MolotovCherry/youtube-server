use anyhow::anyhow;

static CONTENT_TYPES: &[(&str, &str)] = &[
    ("html", "text/html"),
    ("htm", "text/html"),
    ("js", "text/javascript"),
    ("css", "text/css"),
    ("ico", "image/vnd.microsoft.icon"),
    ("txt", "text/plain"),
    ("xml", "application/xml"),
    ("webmanifest", "application/manifest+json"),
    ("png", "image/png"),
    ("svg", "image/svg+xml"),
    ("gif", "image/gif"),
    ("map", "application/json"),
];

pub fn get_content_type(ext: &str) -> anyhow::Result<&'static str> {
    for (_ext, _type) in CONTENT_TYPES {
        if ext == *_ext {
            return Ok(_type);
        }
    }

    Err(anyhow!("Content type not found"))
}
