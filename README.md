# تقویم خورشیدی ویندوز (Windows Jalali Calendar)

یک نرم‌افزار بسیار سبک، سریع و کاملاً بومی (Native) برای سیستم‌عامل ویندوز (سازگار با ویندوز 7، 8، 8.1، 10 و 11) که بدون نیاز به هیچ‌گونه پیش‌نیاز یا ران‌تایم دات‌نت (.NET) نوشته شده است.

---

## 🌟 ویژگی‌های کلیدی (Key Features)

- **آیکون زنده در تسک‌بار (Dynamic System Tray Icon)**:
  - نمایش پویای تاریخ روز جاری (مثلاً `25/5` یا عدد روز با نام ماه).
  - تغییر رنگ هوشمند برای روزهای جمعه و تعطیلات رسمی.
  - تول‌تیپ (Tooltip) غنی با نمایش تاریخ کامل شمسی، میلادی، قمری و رویدادهای روز.
  - تعویض خودکار تاریخ در نیمه‌شب بدون نیاز به مداخله کاربر.

- **پنجره تقویم کامل و مدرن (Modern Full Calendar Popup)**:
  - طراحی مدرن با پشتیبانی کامل از حالت تاریک (Dark Mode) و حالت روشن (Light Mode).
  - چیدمان راست به چپ (RTL) با ارقام زیبای فارسی.
  - ناوبری سریع بین ماه‌ها و سال‌ها + دکمه بازگشت به «امروز».
  - برجسته‌سازی تعطیلات رسمی به رنگ قرمز همراه با نمایش رویدادها و مناسبت‌های رسمی ایران (خورشیدی و قمری).
  - نمایش معادل میلادی و هجری قمری در کارت جزئیات هر روز.
  - ابزار تبدیل تاریخ سه‌گانه (شمسی ↔ میلادی ↔ قمری).

- **فوق‌العاده سبک و کم‌مصرف (Ultra Lightweight)**:
  - حجم فایل اجرایی تنها **~170 کیلوبایت**!
  - مصرف رم بسیار ناچیز (کمتر از ۱۰ مگابایت).
  - بدون نیاز به نصب یا ابزارهای جانبی.
  - پشتیبانی از اجرای خودکار با شروع ویندوز (Start with Windows).

---

## 🚀 نحوه اجرا (How to Run)

فایل اجرایی در مسیر زیر قرار دارد:
```
target\release\wjc.exe
```

1. با اجرای فایل `wjc.exe`، برنامه به صورت خودکار در تسک‌بار (سیستم‌تری) قرار می‌گیرد.
2. با **کلیک چپ** روی آیکون تسک‌بار، تقویم باز یا بسته می‌شود.
3. با **کلیک راست** روی آیکون تسک‌بار، منوی گزینه‌ها (رفتن به امروز، اجرای خودکار با ویندوز، درباره و خروج) ظاهر می‌شود.

---

## 🛠️ کامپایل و بیلد (Build from Source)

برای کامپایل نسخه بهینه‌شده نهایی:
```powershell
cargo build --release
```
فایل خروجی در پوشه `target\release\wjc.exe` تولید می‌شود.


```powershell
cargo test   # official holiday dates 1403–1406 are covered by tests
```

---

## Lunar calendar and holidays

Iran starts each Hijri month by crescent sighting, so purely arithmetic Hijri
calendars drift by a day. The app works like this:

1. [`data/hijri_months.txt`](data/hijri_months.txt) lists the official day 1 of
   every Hijri month (as a Jalali date), currently Shawwal 1445 to Shawwal 1449
   (Farvardin 1403 to Esfand 1406). It is embedded into the exe at build time.
2. Months not listed are estimated astronomically: Meeus new moon plus a minimum
   moon age of 18 hours at Tehran sunset (`src/calendar/astro.rs`). Calibrated
   against the 1403–1406 calendars; may be off by one day.
3. Lunar holidays are fixed Hijri dates (`src/calendar/holidays.rs`), e.g.
   9–10 Muharram, last day of Safar, 1–2 Shawwal. The Hijri day shown in each
   cell uses the same data, so they always agree.

### Updating for a new year (no recompiling)

When the official calendar for a new year is published, create
`hijri_months.txt` **next to `wjc.exe`** with the new month starts and restart
the app. Entries there override the embedded ones:

```
# <hijri year>/<month>  <jalali date of day 1>
1449/11  1407/01/09
1449/12  1407/02/07
```

Tip: the start of a month is the holiday's Jalali date minus (Hijri day − 1),
e.g. Ashura (10 Muharram) on 1407/03/14 means Muharram starts on 1407/03/05.
Entries more than 3 days away from the astronomical estimate are ignored as
typos.
