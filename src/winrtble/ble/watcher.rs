// btleplug Source Code File
//
// Copyright 2020 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.
//
// Some portions of this file are taken and/or modified from Rumble
// (https://github.com/mwylde/rumble), using a dual MIT/Apache License under the
// following copyright:
//
// Copyright (c) 2014 The Rust Project Developers

use crate::{api::ScanFilter, Error, Result};
use windows::{Devices::Bluetooth::Advertisement::*, Foundation::TypedEventHandler};

pub type AdvertismentEventHandler = Box<dyn Fn(&BluetoothLEAdvertisementReceivedEventArgs) + Send>;

pub struct BLEWatcher {
    watcher: BluetoothLEAdvertisementWatcher,
}

impl From<windows::core::Error> for Error {
    fn from(err: windows::core::Error) -> Error {
        Error::Other(format!("{:?}", err).into())
    }
}

impl BLEWatcher {
    pub fn new() -> Self {
        let ad = BluetoothLEAdvertisementFilter::new().unwrap();
        let watcher = BluetoothLEAdvertisementWatcher::Create(&ad).unwrap();
        BLEWatcher { watcher }
    }

    pub fn start(&self, filter: ScanFilter, on_received: AdvertismentEventHandler) -> Result<()> {
        let ScanFilter { services } = filter;

        let services_filter: Vec<windows::core::GUID> = services
            .iter()
            .map(|uuid| windows::core::GUID::from_u128(uuid.as_u128()))
            .collect();

        let handler: TypedEventHandler<BluetoothLEAdvertisementWatcher, BluetoothLEAdvertisementReceivedEventArgs> = TypedEventHandler::new(
            move |_sender, args: &Option<BluetoothLEAdvertisementReceivedEventArgs>| {
                let Some(args) = args else {
                    return Ok(());
                };
                if services_filter.len() == 0 {
                    on_received(args);
                    return Ok(());
                }
                let Ok(advertisement) = args.Advertisement() else {
                    return Ok(());
                };
                let Ok(advertised_services) = advertisement.ServiceUuids() else {
                    return Ok(());
                };
                let Ok(size) = advertised_services.Size() else {
                    return Ok(());
                };
                // Check each service UUID
                for i in 0..size {
                    let Ok(service_guid) = advertised_services.GetAt(i) else {
                        continue;
                    };
                    
                    println!("FOUND SERVICE {:?} at {:?}", service_guid.clone(), i.clone());
                    println!("services_filter {:?}", services_filter);
                    if services_filter.contains(&service_guid.into()) {
                        on_received(args);
                        break;
                    }
                }
                Ok(())
            },
        );
        self.watcher
            .SetScanningMode(BluetoothLEScanningMode::Active)
            .unwrap();
        self.watcher.SetAllowExtendedAdvertisements(true)?;

        self.watcher.Received(&handler)?;
        self.watcher.Start()?;
        Ok(())
    }

    pub fn stop(&self) -> Result<()> {
        self.watcher.Stop()?;
        Ok(())
    }
}
