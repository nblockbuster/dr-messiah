#![feature(random)]
use std::{
    collections::HashMap,
    future,
    io::{Read, Write},
    path::{Path, PathBuf},
    random,
};

use reqwest::header;

#[derive(Debug, Clone)]
pub enum Region {
    America,
}

impl From<Region> for String {
    fn from(region: Region) -> String {
        match region {
            Region::America => "America".to_string(),
        }
    }
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let region: String = self.clone().into();
        write!(f, "{}", region)
    }
}

#[derive(Debug, Clone)]
pub enum Platform {
    Android64,
    Ios,
}

impl From<Platform> for String {
    fn from(platform: Platform) -> String {
        match platform {
            Platform::Android64 => "android64".to_string(),
            Platform::Ios => "ios".to_string(),
        }
    }
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let platform: String = self.clone().into();
        write!(f, "{}", platform)
    }
}

#[derive(Debug, Clone)]
pub enum Quality {
    Emulator,
    High,
    Low,
}

impl From<Quality> for String {
    fn from(quality: Quality) -> String {
        match quality {
            Quality::Emulator => "emulator".to_string(),
            Quality::High => "high".to_string(),
            Quality::Low => "low".to_string(),
        }
    }
}

impl std::fmt::Display for Quality {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let quality: String = self.clone().into();
        write!(f, "{}", quality)
    }
}

#[derive(Debug, Default, Clone, serde::Deserialize)]
pub struct PatchMetadata {
    pub package_version: String,
    pub patch_list_block_urls: Vec<String>,
    pub big_patch_list_block_urls: Vec<String>,
    pub big_patch_2_list_block_urls: Vec<String>,
    pub patch_list_md5: String,
    pub big_patch_list_md5: String,
    pub big_patch_list_2_md5: String,
    pub shader_cache_list_urls: HashMap<String, String>,
    pub hotfix_urls: HashMap<String, String>,
}

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct PatchLists {
    pub patch_list: PatchList,
    pub big_patch_list: BigPatchList,
    pub big_patch_list_2: BigPatchList,
}

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct PatchList {
    pub android64_common: HashMap<String, Vec<serde_json::Value>>,
    pub mapping_version: String,
    pub mpkexclude: Vec<String>,
    pub package_tags_list: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct BigPatchList {
    pub big_android64_common: HashMap<String, Vec<serde_json::Value>>,
    pub big_android_high: Option<HashMap<String, Vec<serde_json::Value>>>,
    pub big_android_emulator: Option<HashMap<String, Vec<serde_json::Value>>>,
    pub big_common: HashMap<String, Vec<serde_json::Value>>,
    pub mapping_version: String,
    pub mapping_ignorelist: Vec<String>,
    pub subtag_bigpatches_group: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct Downloader {
    pub overseas_domain: String,
    pub domestic_domain: String,

    pub game_id: String,
    pub project_id: String,
    pub user_agent: String,

    region: Region,
    platform: Platform,
    quality: Quality,

    orbit_id: String,
    update_host: String,
    gph_host: String,

    metadata: PatchMetadata,
    patch_lists: PatchLists,

    client: reqwest::Client,

    output_path: PathBuf,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    #[serde(rename = "FileSplit")]
    pub file_split: HashMap<String, i64>,
    #[serde(rename = "HostEntry")]
    pub host_entry: HashMap<String, Vec<(String, i64)>>,
    #[serde(rename = "HostResolve")]
    pub host_resolve: HashMap<String, Vec<String>>,
    #[serde(rename = "Resolve")]
    pub resolve: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct JsonPatchMetadata {
    #[serde(rename = "@metadata@")]
    pub metadata: JsonMetadata,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct JsonMetadata {
    #[serde(rename = "@svn_revision@")]
    pub svn_revision: String,
    #[serde(rename = "@svn_branch@")]
    pub svn_branch: String,
    #[serde(rename = "@revision_time@")]
    pub revision_time: String,
    #[serde(rename = "@pkgshaderkey@")]
    pub pkg_shader_key: serde_json::Value,
    #[serde(rename = "@need_hotfix_login_middle@")]
    pub need_hotfix_login_middle: bool,
    #[serde(rename = "@zippedfileblocknum@")]
    pub zipped_file_block_num: i64,
    #[serde(rename = "@bigpatchmd5@")]
    pub big_patch_md5: String,
    #[serde(rename = "@download_low_base_when_android_high@")]
    pub download_low_base_when_android_high: bool,
    #[serde(rename = "@git_revision@")]
    pub git_revision: String,
    #[serde(rename = "@allpatchlistdiffmd5@")]
    pub all_patch_list_diff_md5: serde_json::Value,
    #[serde(rename = "@need_hotfix_login_pre@")]
    pub need_hotfix_login_pre: bool,
    #[serde(rename = "@patch_httpdns_open@")]
    pub patch_http_dns_open: bool,
    #[serde(rename = "@bigpatchmaxratio@")]
    pub big_patch_max_ratio: i64,
    #[serde(rename = "@tinker@")]
    pub tinker: bool,
    #[serde(rename = "@normalpatchthreshold@")]
    pub normal_patch_threshold: i64,
    #[serde(rename = "@bigpatchopen@")]
    pub big_patch_open: bool,
    #[serde(rename = "@need_hotfix_login_post@")]
    pub need_hotfix_login_post: bool,
    #[serde(rename = "@is_inner@")]
    pub is_inner: bool,
    #[serde(rename = "@zippedbigpatchfileblocknum@")]
    pub zipped_big_patch_file_block_num: i64,
    #[serde(rename = "@shadercachelistmd5@")]
    pub shader_cache_list_md5: HashMap<String, String>,
    #[serde(rename = "@zippedbigpatchfileblocknum2@")]
    pub zipped_big_patch_file_block_num_2: i64,
    #[serde(rename = "@bigpatch2md5@")]
    pub big_patch2_md5: String,
}

impl Downloader {
    pub fn new(region: Region, platform: Platform, quality: Quality, output_path: PathBuf) -> Self {
        // orbit id is 16 random bytes as hex
        let orbit_id = random::random::<u128>()
            .to_be_bytes()
            .iter()
            .fold(String::new(), |acc, x| acc + &format!("{:02x}", x));

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("Accept-Encoding", header::HeaderValue::from_static("gzip"));
        headers.insert(
            "X-Ntes-Orbit-ID",
            header::HeaderValue::from_str(&orbit_id).unwrap(),
        );

        Self {
            overseas_domain: "easebar.com".to_string(),
            domestic_domain: "netease.com".to_string(),
            game_id: "g108na".to_string(),
            project_id: "g108naxx2gb".to_string(),
            user_agent: "Orbit/3.6.6 (android 11)".to_string(),

            region,
            platform,
            quality,

            orbit_id,
            update_host: String::new(),
            gph_host: String::new(),

            metadata: PatchMetadata::default(),
            patch_lists: PatchLists::default(),

            client: reqwest::ClientBuilder::new()
                .gzip(true)
                .default_headers(headers)
                .build()
                .expect("Failed to build reqwest client"),

            output_path: output_path.clone(),
        }
    }

    fn get_host_by_type(&self, host_type: &str) -> String {
        format!("{}.{host_type}.{}", self.game_id, self.overseas_domain)
    }

    async fn get(&self, url: &String) -> Result<reqwest::Response, reqwest::Error> {
        // send with header accept gzip + user agent
        self.client.get(url).send().await
    }

    pub async fn fetch_config(&mut self) -> anyhow::Result<(), anyhow::Error> {
        let config_url = format!(
            "https://impression.update.easebar.com/orbit/v3/{}.cfg",
            self.project_id
        );
        let config = self.get(&config_url).await?;
        let conf: Config = config.json().await?;
        println!("{:?}", conf);
        // println!("{:?}", js.get("HostEntry"));

        let mut gph_host = self.get_host_by_type("gph");
        let gph_mirrors = conf.host_entry.get(&gph_host);
        if let Some(mirrors) = gph_mirrors {
            println!("{:?}", mirrors);
            let mut max_metric = -1;
            for (mirror, metric) in mirrors {
                if *metric > max_metric {
                    gph_host = mirror.to_string();
                    max_metric = *metric;
                }
            }
        }

        self.update_host = self.get_host_by_type("update");
        self.gph_host = gph_host;

        // self._session.headers.update(
        //     {"Accept-Encoding": "identity", "X-Ntes-Orbit-ID": self._orbit_id}
        // )

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "Accept-Encoding",
            header::HeaderValue::from_static("identity"),
        );
        headers.insert(
            "X-Ntes-Orbit-ID",
            header::HeaderValue::from_str(&self.orbit_id).unwrap(),
        );

        self.client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .build()
            .expect("Failed to build reqwest client");

        Ok(())
    }

    pub async fn fetch_metadata(&mut self) -> anyhow::Result<(), anyhow::Error> {
        let patch_md5_url = format!(
            "https://{}/pl/{}_patchmd5_{}_{}.txt",
            self.update_host, self.region, self.platform, self.quality
        );
        let response = self.get(&patch_md5_url).await?;
        // split everything before the json data ({)
        let patch_md5 = response.text().await?;

        let before_json = patch_md5.split("{").collect::<Vec<&str>>()[0]
            .split("\n")
            .collect::<Vec<&str>>();
        // everything after the first { inclusive, cant split because multiple
        let json_data = patch_md5.replace(before_json.join("\n").as_str(), "");

        //println!("{:#?}", before_json);

        let pkg_ver = before_json[0];
        let patch_list_md5 = before_json[1];

        println!("{:?}", pkg_ver);
        println!("{:?}", patch_list_md5);

        // println!("{:?}", json_data);

        let metadata: JsonPatchMetadata = serde_json::from_str(&json_data)?;

        println!("{:?}", metadata.metadata);
        let mut patch_list_blocks = Vec::new();
        let mut big_patch_list_blocks = Vec::new();
        let mut big2_patch_list_blocks = Vec::new();
        let mut shader_cache_lists = HashMap::new();

        for i in 0..metadata.metadata.zipped_file_block_num {
            let patch_list_url = format!(
                "https://{}/{}/{pkg_ver}/AdditionalInfo/{}_{}/patchlist_{}_{}_zipped_block_{}.txt",
                self.gph_host,
                self.region,
                self.platform,
                self.quality,
                self.platform,
                self.quality,
                i
            );
            patch_list_blocks.push(patch_list_url);
        }

        for i in 0..metadata.metadata.zipped_big_patch_file_block_num {
            let big_patch_list_url = format!(
                "https://{}/{}/big_{pkg_ver}/AdditionalInfo/{}_{}/bigpatchlist_{}_{}_zipped_block_{}.txt",
                self.gph_host,
                self.region,
                self.platform,
                self.quality,
                self.platform,
                self.quality,
                i
            );
            big_patch_list_blocks.push(big_patch_list_url);
        }

        for i in 0..metadata.metadata.zipped_big_patch_file_block_num_2 {
            let big_patch_list_url = format!(
                "https://{}/{}/big2_{pkg_ver}/AdditionalInfo/{}_{}/bigpatch2list_{}_{}_zipped_block_{}.txt",
                self.gph_host,
                self.region,
                self.platform,
                self.quality,
                self.platform,
                self.quality,
                i
            );
            big2_patch_list_blocks.push(big_patch_list_url);
        }

        for (key, value) in metadata.metadata.shader_cache_list_md5 {
            let shader_cache_list_url =
                format!("https://{}/{}/{pkg_ver}/{key}", self.gph_host, self.region);
            shader_cache_lists.insert(shader_cache_list_url, value);
        }

        println!("{:?}", patch_list_blocks);
        println!("{:?}", big_patch_list_blocks);
        println!("{:?}", shader_cache_lists);

        let mut hotfixes: HashMap<String, String> = HashMap::new();

        if metadata.metadata.need_hotfix_login_pre {
            hotfixes.insert(
                String::from("pre"),
                format!(
                    "https://{}/pl/{}_hotfix_login_pre_file.txt",
                    self.update_host, self.region
                ),
            );
        }

        if metadata.metadata.need_hotfix_login_middle {
            hotfixes.insert(
                String::from("middle"),
                format!(
                    "https://{}/pl/{}_hotfix_login_middle_file.txt",
                    self.update_host, self.region
                ),
            );
        }

        if metadata.metadata.need_hotfix_login_post {
            hotfixes.insert(
                String::from("post"),
                format!(
                    "https://{}/pl/{}_hotfix_login_post_file.txt",
                    self.update_host, self.region
                ),
            );
        }

        self.metadata = PatchMetadata {
            package_version: pkg_ver.to_string(),
            patch_list_block_urls: patch_list_blocks,
            big_patch_list_block_urls: big_patch_list_blocks,
            patch_list_md5: patch_list_md5.to_string(),
            big_patch_list_md5: metadata.metadata.big_patch_md5,
            shader_cache_list_urls: shader_cache_lists,
            hotfix_urls: hotfixes,
            big_patch_2_list_block_urls: big2_patch_list_blocks,
            big_patch_list_2_md5: metadata.metadata.big_patch2_md5,
        };

        Ok(())
    }

    pub async fn download_patch_lists(&mut self) -> anyhow::Result<(), anyhow::Error> {
        let base_path = self.output_path.join(format!(
            "{}/{}/AdditionalInfo/{}_{}",
            self.region, self.metadata.package_version, self.platform, self.quality
        ));

        std::fs::create_dir_all(&base_path)?;
        // need to concat the patch list blocks into one file
        let mut patch_list = Vec::new();
        for url in self.metadata.patch_list_block_urls.iter() {
            let response = self.get(&url.clone()).await?;
            let data = response.bytes().await?.to_vec();
            patch_list.push(data);
        }

        let patchlist_zip: Vec<u8> = patch_list.concat();
        let mut patchlist_raw = Vec::new();
        flate2::read::ZlibDecoder::new(&patchlist_zip[..]).read_to_end(&mut patchlist_raw)?;

        let patchlist_md5 = String::from_utf8_lossy(&md5::compute(&patchlist_raw).0).to_string();

        let patchlist: PatchList = serde_json::from_slice(&patchlist_raw)?;
        let patchlist_name = format!("patchlist_{}_{}.json", self.platform, self.quality);
        let mut patchlist_file = std::fs::File::create(base_path.join(patchlist_name))?;
        patchlist_file.write_all(serde_json::to_string_pretty(&patchlist)?.as_bytes())?;

        if patchlist_md5 != self.metadata.patch_list_md5 {
            println!("Patchlist md5 mismatch");
        }

        // bigpatchlist

        let mut big_patch_list = Vec::new();
        for url in self.metadata.big_patch_list_block_urls.iter() {
            let response = self.get(&url.clone()).await?;
            let data = response.bytes().await?.to_vec();
            big_patch_list.push(data);
        }

        let big_patchlist_zip: Vec<u8> = big_patch_list.concat();
        let mut big_patchlist_raw = Vec::new();
        flate2::read::ZlibDecoder::new(&big_patchlist_zip[..])
            .read_to_end(&mut big_patchlist_raw)?;

        let big_patchlist_md5 =
            String::from_utf8_lossy(&md5::compute(&big_patchlist_raw).0).to_string();

        let big_patchlist: BigPatchList = serde_json::from_slice(&big_patchlist_raw)?;
        let big_patchlist_name = format!("bigpatchlist_{}_{}.json", self.platform, self.quality);
        let mut big_patchlist_file = std::fs::File::create(base_path.join(big_patchlist_name))?;
        big_patchlist_file.write_all(serde_json::to_string_pretty(&big_patchlist)?.as_bytes())?;

        if big_patchlist_md5 != self.metadata.big_patch_list_md5 {
            println!("Big patchlist md5 mismatch");
        }

        let mut big_patch_list_2 = Vec::new();
        for url in self.metadata.big_patch_2_list_block_urls.iter() {
            let response = self.get(&url.clone()).await?;
            let data = response.bytes().await?.to_vec();
            big_patch_list_2.push(data);
        }

        let big_patchlist_2_zip: Vec<u8> = big_patch_list_2.concat();
        let mut big_patchlist_2_raw = Vec::new();
        flate2::read::ZlibDecoder::new(&big_patchlist_zip[..])
            .read_to_end(&mut big_patchlist_2_raw)?;

        let big_patchlist_2_md5 =
            String::from_utf8_lossy(&md5::compute(&big_patchlist_2_raw).0).to_string();

        let big_patchlist2: BigPatchList = serde_json::from_slice(&big_patchlist_2_raw)?;
        let big_patchlist2_name = format!("bigpatchlist2_{}_{}.json", self.platform, self.quality);
        let mut big_patchlist2_file = std::fs::File::create(base_path.join(big_patchlist2_name))?;
        big_patchlist2_file.write_all(serde_json::to_string_pretty(&big_patchlist2)?.as_bytes())?;

        if big_patchlist_2_md5 != self.metadata.big_patch_list_2_md5 {
            println!("Big patchlist md5 mismatch");
        }

        self.patch_lists = PatchLists {
            patch_list: patchlist,
            big_patch_list: big_patchlist,
            big_patch_list_2: big_patchlist2,
        };

        Ok(())
    }

    // TODO: make this better
    pub async fn download_files(&self) -> anyhow::Result<(), anyhow::Error> {
        let tasks: Vec<_> = self
            .patch_lists
            .patch_list
            .android64_common
            .iter()
            .map(|(key, value)| {
                let k = key.clone();
                let v = value.clone();
                let gph_host = self.gph_host.clone();
                let region = self.region.clone();
                let patch_lists = self.patch_lists.clone();
                let output_path = self.output_path.clone().join(format!(
                    "{}/{}/{}",
                    self.region, self.platform, self.quality
                ));
                tokio::spawn(async move {
                    let path = format!(
                        "https://{}/{}/{}/android64_common/{}", //todo: find way of doing android64_common automatically, video files are in common instead
                        gph_host, region, patch_lists.patch_list.mapping_version, k
                    );
                    if k.contains("huge_split") {
                        return;
                    }
                    println!("{:?}", path);

                    let response = reqwest::get(&path).await.unwrap();
                    let data = response.bytes().await.unwrap().to_vec();

                    let mut fpath = output_path.join(&k);
                    if fpath.extension().is_none() {
                        fpath.set_extension("bin");
                    }

                    std::fs::create_dir_all(fpath.parent().unwrap()).unwrap();
                    let mut file = std::fs::File::create(fpath).unwrap();
                    file.write_all(&data).unwrap();

                    let md5 = md5::compute(&data);
                    let md5_str = format!("{:x}", md5);
                    if md5_str != v.first().unwrap().as_str().unwrap() {
                        println!("MD5 mismatch for {}", &k);
                    }
                })
            })
            .collect();

        futures::future::join_all(tasks).await;

        if let Some(big_list) = &self.patch_lists.big_patch_list.big_android_high {
            let tasks: Vec<_> = big_list
                .iter()
                .map(|(key, value)| {
                    let k = key.clone();
                    let v = value.clone();
                    let gph_host = self.gph_host.clone();
                    let region = self.region.clone();
                    let patch_lists = self.patch_lists.clone();
                    let output_path = self.output_path.clone().join(format!(
                        "{}/{}/{}",
                        self.region, self.platform, self.quality
                    ));
                    tokio::spawn(async move {
                        let path = format!(
                            "https://{}/{}/big_{}/big_android_high/{}", //todo: find way of doing android64_common automatically, video files are in common instead
                            gph_host, region, patch_lists.patch_list.mapping_version, k
                        );

                        println!("{:?}", path);

                        let response = reqwest::get(&path).await.unwrap();
                        let data = response.bytes().await.unwrap().to_vec();

                        let mut fpath = output_path.join(&k);
                        if fpath.extension().is_none() {
                            fpath.set_extension("bin");
                        }

                        std::fs::create_dir_all(fpath.parent().unwrap()).unwrap();
                        let mut file = std::fs::File::create(fpath).unwrap();
                        file.write_all(&data).unwrap();

                        let md5 = md5::compute(&data);
                        let md5_str = format!("{:x}", md5);
                        if md5_str != v.first().unwrap().as_str().unwrap() {
                            println!("MD5 mismatch for {}", &k);
                        }
                    })
                })
                .collect();

            futures::future::join_all(tasks).await;
        }

        if let Some(big_list) = &self.patch_lists.big_patch_list.big_android_emulator {
            let tasks: Vec<_> = big_list
                .iter()
                .map(|(key, value)| {
                    let k = key.clone();
                    let v = value.clone();
                    let gph_host = self.gph_host.clone();
                    let region = self.region.clone();
                    let patch_lists = self.patch_lists.clone();
                    let output_path = self.output_path.clone().join(format!(
                        "{}/{}/{}",
                        self.region, self.platform, self.quality
                    ));
                    tokio::spawn(async move {
                        let path = format!(
                            "https://{}/{}/big_{}/big_android_emulator/{}", //todo: find way of doing android64_common automatically, video files are in common instead
                            gph_host, region, patch_lists.patch_list.mapping_version, k
                        );

                        println!("{:?}", path);

                        let response = reqwest::get(&path).await.unwrap();
                        let data = response.bytes().await.unwrap().to_vec();

                        let mut fpath = output_path.join(&k);
                        if fpath.extension().is_none() {
                            fpath.set_extension("bin");
                        }

                        std::fs::create_dir_all(fpath.parent().unwrap()).unwrap();
                        let mut file = std::fs::File::create(fpath).unwrap();
                        file.write_all(&data).unwrap();

                        let md5 = md5::compute(&data);
                        let md5_str = format!("{:x}", md5);
                        if md5_str != v.first().unwrap().as_str().unwrap() {
                            println!("MD5 mismatch for {}", &k);
                        }
                    })
                })
                .collect();

            futures::future::join_all(tasks).await;
        }

        // if let Some(big_list) = &self.patch_lists.big_patch_list_2.big_android_high {
        //     let tasks: Vec<_> = big_list
        //         .iter()
        //         .map(|(key, value)| {
        //             let k = key.clone();
        //             let v = value.clone();
        //             let gph_host = self.gph_host.clone();
        //             let region = self.region.clone();
        //             let patch_lists = self.patch_lists.clone();
        //             let output_path = self.output_path.clone().join(format!(
        //                 "{}/{}/{}",
        //                 self.region, self.platform, self.quality
        //             ));
        //             tokio::spawn(async move {
        //                 let path = format!(
        //                     "https://{}/{}/big_{}/big2_android_high/{}", //todo: find way of doing android64_common automatically, video files are in common instead
        //                     gph_host, region, patch_lists.patch_list.mapping_version, k
        //                 );
        //                 println!("{:?}", path);
        //                 let response = reqwest::get(&path).await.unwrap();
        //                 let data = response.bytes().await.unwrap().to_vec();
        //                 let mut fpath = output_path.join(&k);
        //                 if fpath.extension().is_none() {
        //                     fpath.set_extension("bin");
        //                 }
        //                 std::fs::create_dir_all(fpath.parent().unwrap()).unwrap();
        //                 let mut file = std::fs::File::create(fpath).unwrap();
        //                 file.write_all(&data).unwrap();

        //                 let md5 = md5::compute(&data);
        //                 let md5_str = format!("{:x}", md5);
        //                 if md5_str != v.first().unwrap().as_str().unwrap() {
        //                     println!("MD5 mismatch for {}", &k);
        //                 }
        //             })
        //         })
        //         .collect();
        //     futures::future::join_all(tasks).await;
        // }

        // if let Some(big_list) = &self.patch_lists.big_patch_list_2.big_android_emulator {
        //     let tasks: Vec<_> = big_list
        //         .iter()
        //         .map(|(key, value)| {
        //             let k = key.clone();
        //             let v = value.clone();
        //             let gph_host = self.gph_host.clone();
        //             let region = self.region.clone();
        //             let patch_lists = self.patch_lists.clone();
        //             let output_path = self.output_path.clone().join(format!(
        //                 "{}/{}/{}",
        //                 self.region, self.platform, self.quality
        //             ));
        //             tokio::spawn(async move {
        //                 let path = format!(
        //                     "https://{}/{}/big2_{}/big2_android_emulator/{}", //todo: find way of doing android64_common automatically, video files are in common instead
        //                     gph_host, region, patch_lists.patch_list.mapping_version, k
        //                 );
        //                 println!("{:?}", path);
        //                 let response = reqwest::get(&path).await.unwrap();
        //                 let data = response.bytes().await.unwrap().to_vec();
        //                 let mut fpath = output_path.join(&k);
        //                 if fpath.extension().is_none() {
        //                     fpath.set_extension("bin");
        //                 }
        //                 std::fs::create_dir_all(fpath.parent().unwrap()).unwrap();
        //                 let mut file = std::fs::File::create(fpath).unwrap();
        //                 file.write_all(&data).unwrap();
        //                 let md5 = md5::compute(&data);
        //                 let md5_str = format!("{:x}", md5);
        //                 if md5_str != v.first().unwrap().as_str().unwrap() {
        //                     println!("MD5 mismatch for {}", &k);
        //                 }
        //             })
        //         })
        //         .collect();
        //     futures::future::join_all(tasks).await;
        // }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<(), anyhow::Error> {
    let mut downloader = Downloader::new(
        Region::America,
        Platform::Android64,
        Quality::Emulator,
        PathBuf::from("G:\\DestinyRising\\patch_download_beta"),
    );
    downloader.fetch_config().await?;
    downloader.fetch_metadata().await?;
    downloader.download_patch_lists().await?;

    // downloader.download_files().await?;

    // println!("{:#?}", downloader.metadata);
    Ok(())
}
