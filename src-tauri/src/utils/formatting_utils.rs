// Copyright 2024. The Tari Project
//
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
// following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
// disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
// following disclaimer in the documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
// products derived from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
// INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
// WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
// USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

pub fn format_hashrate(hashrate: f64) -> String {
    if hashrate < 1000.0 {
        format!("{hashrate:.2} H/s")
    } else if hashrate < 1_000_000.0 {
        format!("{:.2} kH/s", hashrate / 1000.0)
    } else if hashrate < 1_000_000_000.0 {
        format!("{:.2} MH/s", hashrate / 1_000_000.0)
    } else if hashrate < 1_000_000_000_000.0 {
        format!("{:.2} GH/s", hashrate / 1_000_000_000.0)
    } else if hashrate < 1_000_000_000_000_000.0 {
        format!("{:.2} TH/s", hashrate / 1_000_000_000_000.0)
    } else {
        format!("{:.2} PH/s", hashrate / 1_000_000_000_000_000.0)
    }
}

pub fn format_currency(balance: f64, currency: &str) -> String {
    if balance < 0.0 {
        // Handle negative balances
        return format!("-{}", format_currency(-balance, currency));
    }

    if balance < 1000.0 {
        format!("{balance:.2} {currency}")
    } else if balance < 1_000_000.0 {
        format!("{:.2}k {}", balance / 1000.0, currency)
    } else if balance < 1_000_000_000.0 {
        format!("{:.2}m {}", balance / 1_000_000.0, currency)
    } else if balance < 1_000_000_000_000.0 {
        format!("{:.2}b {}", balance / 1_000_000_000.0, currency)
    } else if balance < 1_000_000_000_000_000.0 {
        format!("{:.2}t {}", balance / 1_000_000_000_000.0, currency)
    } else {
        format!("{:.2}q {}", balance / 1_000_000_000_000_000.0, currency)
    }
}
