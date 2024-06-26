
use anyhow::bail;
use esp_idf_svc::{
    http::client::{Configuration, EspHttpConnection},
};

use embedded_svc::{
    http::{client::Client, Method},
    io::Read,
};
use anyhow::Result;
use core::str;
pub fn get(url: impl AsRef<str>) -> Result<String> {
    // 1. Create a new EspHttpConnection with default Configuration. (Check documentation)
    let connection = EspHttpConnection::new(&Configuration {
        use_global_ca_store: true,
        crt_bundle_attach: Some(esp_idf_svc::sys::esp_crt_bundle_attach),
        ..Default::default()
    })?;
    // 2. Get a client using the embedded_svc Client::wrap method. (Check documentation)
    let mut client = Client::wrap(connection);

    // 3. Open a GET request to `url`
    let headers = [("accept", "text/plain")];
    // ANCHOR: request
    let request = client.request(Method::Get, url.as_ref(), &headers)?;
    // ANCHOR_END: request

    // 4. Submit the request and check the status code of the response.
    // Successful http status codes are in the 200..=299 range.
    let response = request.submit()?;
    let status = response.status();
    let mut result = String::new();
    println!("Response code: {}\n", status);
    match status {
        200..=299 => {
            // 5. If the status is OK, read response data chunk by chunk into a buffer and print it until done.
            //
            // NB. There is no guarantee that chunks will be split at the boundaries of valid UTF-8
            // sequences (in fact it is likely that they are not) so this edge case needs to be handled.
            // However, for the purposes of clarity and brevity(?), the additional case of completely invalid
            // UTF-8 sequences will not be handled here and is left as an exercise for the reader.
            let mut buf = [0_u8; 256];
            // Offset into the buffer to indicate that there may still be
            // bytes at the beginning that have not been decoded yet
            let mut offset = 0;
            // Keep track of the total number of bytes read to print later
            let mut total = 0;
            let mut reader = response;
            loop {
                // read into the buffer starting at the offset to not overwrite
                // the incomplete UTF-8 sequence we put there earlier
                if let Ok(size) = Read::read(&mut reader, &mut buf[offset..]) {
                    if size == 0 {
                        // It might be nice to check if we have any left over bytes here (ie. the offset > 0)
                        // as this would mean that the response ended with an invalid UTF-8 sequence, but for the
                        // purposes of this training we are assuming that the full response will be valid UTF-8
                        break;
                    }
                    // Update the total number of bytes read
                    total += size;
                    // 6. Try converting the bytes into a Rust (UTF-8) string and print it.
                    // Remember that we read into an offset and recalculate the real length
                    // of the bytes to decode.
                    let size_plus_offset = size + offset;
                    match str::from_utf8(&buf[..size_plus_offset]) {
                        Ok(text) => {
                            // buffer contains fully valid UTF-8 data,
                            // print it and reset the offset to 0.
                            result.push_str(text);

                            offset = 0;
                        }
                        Err(error) => {
                            println!("error: {}\n", error);
                            // The buffer contains incomplete UTF-8 data, we will
                            // print the valid part, copy the invalid sequence to
                            // the beginning of the buffer and set an offset for the
                            // next read.
                            //
                            // NB. There is actually an additional case here that should be
                            // handled in a real implementation. The Utf8Error may also contain
                            // an error_len field indicating that there is actually an invalid UpTF-8
                            // sequence in the middle of the buffer. Such an error would not be
                            // recoverable through our offset and copy mechanism. The result will be
                            // that the invalid sequence will be copied to the front of the buffer and
                            // eventually the buffer will be filled until no more bytes can be read when
                            // the offset == buf.len(). At this point the loop will exit without reading
                            // any more of the response.
                            let valid_up_to = error.valid_up_to();
                            unsafe {
                                // It's ok to use unsafe here as the error code already told us that
                                // the UTF-8 data up to this point is valid, so we can tell the compiler
                                // it's fine.
                                print!("{}", str::from_utf8_unchecked(&buf[..valid_up_to]));
                            }
                            buf.copy_within(valid_up_to.., 0);
                            offset = size_plus_offset - valid_up_to;
                        }
                    }
                }
            }
            println!("Total: {} bytes", total);
        }
        _ => bail!("Unexpected response code: {}", status),
    }

    Ok(result)
}
