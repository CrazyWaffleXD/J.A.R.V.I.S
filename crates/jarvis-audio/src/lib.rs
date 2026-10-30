//! Захват аудио с микрофона.
//!
//! Синтез речи и распознавание остаются в других крейтах: здесь только
//! запись моно-сэмплов `f32` в диапазоне `[-1.0, 1.0]`. Реализация
//! воспроизведения на `rodio` появится позже.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, SizedSample, StreamConfig};

/// Уровень RMS, ниже которого сигнал считается тишиной.
const SILENCE_RMS: f32 = 0.01;

/// Как часто проверять буфер на тишину.
const POLL_INTERVAL: Duration = Duration::from_millis(50);

/// Записать с микрофона, пока не наступит тишина `silence_ms` или не
/// пройдёт `max_ms`.
///
/// Возвращает моно-сэмплы `f32` в диапазоне `[-1.0, 1.0]`, приведённые
/// к `sample_rate`. Тишина определяется простым порогом по RMS, без
/// VAD-модели.
pub fn record_until_silence(sample_rate: u32, silence_ms: u64, max_ms: u64) -> Result<Vec<f32>> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| anyhow!("микрофон не найден: нет устройства ввода по умолчанию"))?;

    let supported = device
        .default_input_config()
        .map_err(|e| anyhow!("не удалось получить формат микрофона: {e}"))?;
    let device_rate = supported.sample_rate().0;
    let config: StreamConfig = supported.config();

    let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));

    let stream = match supported.sample_format() {
        SampleFormat::F32 => build_stream::<f32>(&device, &config, Arc::clone(&buffer))?,
        SampleFormat::I16 => build_stream::<i16>(&device, &config, Arc::clone(&buffer))?,
        SampleFormat::U16 => build_stream::<u16>(&device, &config, Arc::clone(&buffer))?,
        format => {
            return Err(anyhow!(
                "неподдерживаемый формат сэмплов микрофона: {format:?}"
            ))
        }
    };

    stream
        .play()
        .map_err(|e| anyhow!("не удалось запустить микрофон: {e}"))?;

    let started = Instant::now();
    let mut last_check = started;
    let mut silence = Duration::ZERO;
    let mut consumed = 0usize;

    loop {
        std::thread::sleep(POLL_INTERVAL);

        let chunk = {
            let buffer = buffer
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            buffer[consumed..].to_vec()
        };
        consumed += chunk.len();

        if rms(&chunk) < SILENCE_RMS {
            silence += last_check.elapsed();
        } else {
            silence = Duration::ZERO;
        }
        last_check = Instant::now();

        let silence_reached = silence.as_millis() as u64 >= silence_ms;
        let time_exceeded = started.elapsed().as_millis() as u64 >= max_ms;
        if silence_reached || time_exceeded {
            break;
        }
    }

    let _ = stream.pause();
    drop(stream);

    let samples = std::mem::take(
        &mut *buffer
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
    );
    if samples.is_empty() {
        return Err(anyhow!("микрофон не вернул ни одного сэмпла"));
    }

    Ok(resample(&samples, device_rate, sample_rate))
}

/// Собрать входной поток и писать моно-сэмплы `f32` в общий буфер.
fn build_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    buffer: Arc<Mutex<Vec<f32>>>,
) -> Result<cpal::Stream>
where
    T: SizedSample + Sample<Float = f32>,
{
    let channels = config.channels as usize;
    let stream = device
        .build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                let mut buffer = buffer
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                for frame in data.chunks(channels) {
                    let sum: f32 = frame.iter().map(|sample| sample.to_float_sample()).sum();
                    buffer.push(sum / channels as f32);
                }
            },
            |err| tracing::warn!(%err, "ошибка входного аудиопотока"),
            None,
        )
        .map_err(|e| anyhow!("не удалось открыть микрофон: {e}"))?;

    Ok(stream)
}

/// Линейная передискретизация моно-сигнала.
fn resample(samples: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to || from == 0 || to == 0 || samples.is_empty() {
        return samples.to_vec();
    }

    let ratio = from as f64 / to as f64;
    let out_len = (samples.len() as f64 / ratio).round() as usize;
    let mut out = Vec::with_capacity(out_len);

    for i in 0..out_len {
        let pos = i as f64 * ratio;
        let index = pos.floor() as usize;
        let frac = (pos - index as f64) as f32;
        let a = samples[index.min(samples.len() - 1)];
        let b = samples[(index + 1).min(samples.len() - 1)];
        out.push(a + (b - a) * frac);
    }

    out
}

/// Среднеквадратичный уровень сигнала.
fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }

    (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
}
