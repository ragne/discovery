use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Error as FmtError, Formatter};

#[derive(Debug, Clone, Serialize)]
pub struct RegisterRequest<'reg> {
    pub instance: &'reg Instance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub host_name: String,
    pub app: String,
    pub ip_addr: String,
    pub vip_address: String,
    pub secure_vip_address: Option<String>,
    pub status: StatusType,
    pub port: Option<PortData>,
    pub secure_port: PortData,
    pub home_page_url: String,
    pub status_page_url: String,
    pub health_check_url: String,
    pub data_center_info: DataCenterInfo,
    pub lease_info: Option<LeaseInfo>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortData {
    #[serde(rename = "$")]
    value: u16,
    #[serde(
        rename = "@enabled",
        deserialize_with = "deserializers::bool_from_string"
    )]
    enabled: bool,
}

impl PortData {
    pub fn new(port: u16, enabled: bool) -> Self {
        PortData {
            value: port,
            enabled,
        }
    }

    pub fn value(&self) -> Option<u16> {
        self.enabled.then(|| self.value)
    }
}

#[derive(Debug, Clone)]
pub struct Applications {
    pub applications: HashMap<String, Vec<Instance>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCenterInfo {
    #[serde(rename = "@class")]
    class: String,
    pub name: DcNameType,
    /// metadata is only allowed if name is Amazon, and then is required
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<AmazonMetadataType>,
}

impl Default for DataCenterInfo {
    fn default() -> Self {
        DataCenterInfo {
            class: "com.netflix.appinfo.InstanceInfo$DefaultDataCenterInfo".into(),
            name: DcNameType::MyOwn,
            metadata: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaseInfo {
    /// (optional) if you want to change the length of lease - default if 90 secs
    pub eviction_duration_in_secs: Option<usize>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DcNameType {
    MyOwn,
    Amazon,
}

impl Display for DcNameType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StatusType {
    Up,
    Down,
    Starting,
    OutOfService,
    Unknown,
}

impl Display for StatusType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        write!(
            f,
            "{}",
            serde_json::to_value(self).unwrap().as_str().unwrap()
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AmazonMetadataType {
    // old versions do not have that field
    pub ami_launch_index: Option<String>,
    pub local_hostname: String,
    pub availability_zone: String,
    pub instance_id: String,
    // instance might not have one
    pub public_ipv4: Option<String>,
    // same with this
    pub public_hostname: Option<String>,
    // old versions do not have that field
    pub ami_manifest_path: Option<String>,
    pub local_ipv4: String,
    // old versions do not have that field
    pub hostname: Option<String>,
    pub ami_id: String,
    pub instance_type: String,
}

impl<'de> Deserialize<'de> for Applications {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Debug)]
        struct AppWithName {
            name: String,
            instance: Vec<Instance>,
        }

        #[derive(Deserialize, Debug)]
        struct Apps {
            application: Vec<AppWithName>,
        }

        #[derive(Deserialize, Debug)]
        struct Outer {
            applications: Apps,
        }

        let helper = Outer::deserialize(deserializer)?.applications;
        Ok(Applications {
            applications: helper
                .application
                .into_iter()
                .map(|app| (app.name, app.instance))
                .collect(),
        })
    }
}

pub(crate) mod deserializers {
    use serde::{
        de::{self, Unexpected},
        Deserialize, Deserializer,
    };

    /// Deserialize bool from String with custom value mapping
    pub fn bool_from_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer)?.as_ref() {
            "true" => Ok(true),
            "false" => Ok(false),
            other => Err(de::Error::invalid_value(
                Unexpected::Str(other),
                &"true or false",
            )),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    const REG: &str = r#"{
        "applications": {
          "versions__delta": "1",
          "apps__hashcode": "UP_2_",
          "application": [
            {
              "name": "A-BOOTIFUL-CLIENT",
              "instance": [
                {
                  "instanceId": "172.16.200.36:a-bootiful-client:8082",
                  "hostName": "172.16.200.36",
                  "app": "A-BOOTIFUL-CLIENT",
                  "ipAddr": "172.16.200.36",
                  "status": "UP",
                  "overriddenStatus": "UNKNOWN",
                  "port": {
                    "$": 8082,
                    "@enabled": "true"
                  },
                  "securePort": {
                    "$": 443,
                    "@enabled": "false"
                  },
                  "countryId": 1,
                  "dataCenterInfo": {
                    "@class": "com.netflix.appinfo.InstanceInfo$DefaultDataCenterInfo",
                    "name": "MyOwn"
                  },
                  "leaseInfo": {
                    "renewalIntervalInSecs": 30,
                    "durationInSecs": 90,
                    "registrationTimestamp": 1623337208578,
                    "lastRenewalTimestamp": 1623337208578,
                    "evictionTimestamp": 0,
                    "serviceUpTimestamp": 1623337208067
                  },
                  "metadata": {
                    "management.port": "8082"
                  },
                  "homePageUrl": "http://172.16.200.36:8082/",
                  "statusPageUrl": "http://172.16.200.36:8082/actuator/info",
                  "healthCheckUrl": "http://172.16.200.36:8082/actuator/health",
                  "vipAddress": "a-bootiful-client",
                  "secureVipAddress": "a-bootiful-client",
                  "isCoordinatingDiscoveryServer": "false",
                  "lastUpdatedTimestamp": "1623337208578",
                  "lastDirtyTimestamp": "1623337208042",
                  "actionType": "ADDED"
                },
                {
                  "instanceId": "172.16.200.36:a-bootiful-client",
                  "hostName": "172.16.200.36",
                  "app": "A-BOOTIFUL-CLIENT",
                  "ipAddr": "172.16.200.36",
                  "status": "UP",
                  "overriddenStatus": "UNKNOWN",
                  "port": {
                    "$": 8080,
                    "@enabled": "true"
                  },
                  "securePort": {
                    "$": 443,
                    "@enabled": "false"
                  },
                  "countryId": 1,
                  "dataCenterInfo": {
                    "@class": "com.netflix.appinfo.InstanceInfo$DefaultDataCenterInfo",
                    "name": "MyOwn"
                  },
                  "leaseInfo": {
                    "renewalIntervalInSecs": 30,
                    "durationInSecs": 90,
                    "registrationTimestamp": 1623337120451,
                    "lastRenewalTimestamp": 1623337300467,
                    "evictionTimestamp": 0,
                    "serviceUpTimestamp": 1623337119939
                  },
                  "metadata": {
                    "management.port": "8080"
                  },
                  "homePageUrl": "http://172.16.200.36:8080/",
                  "statusPageUrl": "http://172.16.200.36:8080/actuator/info",
                  "healthCheckUrl": "http://172.16.200.36:8080/actuator/health",
                  "vipAddress": "a-bootiful-client",
                  "secureVipAddress": "a-bootiful-client",
                  "isCoordinatingDiscoveryServer": "false",
                  "lastUpdatedTimestamp": "1623337120451",
                  "lastDirtyTimestamp": "1623337119936",
                  "actionType": "ADDED"
                }
              ]
            }
          ]
        }
      }"#;

    #[test]
    fn test_de() {
        let r: Applications = serde_json::from_str(&REG).unwrap();
        println!("{:#?}", r);
        assert!(r.applications.get("A-BOOTIFUL-CLIENT").unwrap().len() == 2);
    }
}
