# تقویم خورشیدی ویندوز (Windows Jalali Calendar)

یک نرم‌افزار بسیار سبک، سریع و کاملاً بومی (Native) برای سیستم‌عامل ویندوز (سازگار با ویندوز 7، 8، 8.1، 10 و 11) که بدون نیاز به هیچ‌گونه پیش‌نیاز یا ران‌تایم دات‌نت (.NET) نوشته شده است.

---

## 🌟 ویژگی‌های کلیدی (Key Features)

- **آیکون زنده در تسک‌بار (Dynamic System Tray Icon)**:
  - رندر پیکسلی شفاف و دقیق متناسب با DPI واقعی سیستم (`16px` تا `64px`) همراه با پشتیبانی از هر دو تم روشن و تاریک ویندوز.
  - سه حالت قابل انتخاب از منوی کلیک راست: **خودکار** (عدد روز بزرگ در ۱۶ پیکسل و نام ماه + روز در ۲۰ پیکسل به بالا)، **نام ماه و روز**، و **عددی (`۲۵/۵`)**.
  - تغییر رنگ هوشمند برای روزهای جمعه و تعطیلات رسمی + بازیابی خودکار پس از ریستارت Explorer.
  - تول‌تیپ (Tooltip) غنی با نمایش تاریخ کامل شمسی، میلادی، قمری و رویدادهای روز.
  - تعویض خودکار و آنی تاریخ و تم (`WM_SETTINGCHANGE` و `WM_TIMECHANGE`).

- **پنجره تقویم کامل و مدرن (Modern Full Calendar Popup)**:
  - طراحی تمیز با پشتیبانی از حالت تاریک و روشن (هماهنگ با تم ویندوز و ذخیره تنظیمات کاربر) و گوشه‌های گرد بومی ویندوز ۱۱.
  - چیدمان راست به چپ (RTL) با ارقام فارسی و آیکون‌های برداری بومی بدون ایموجی.
  - ناوبری سریع با دکمه‌ها، چرخ ماوس (Mouse Wheel)، کلیدهای جهت‌نما، `PageUp`/`PageDown`، `Home` و کلیک روی روزهای ماه قبل/بعد.
  - برجسته‌سازی تعطیلات رسمی به رنگ قرمز همراه با نمایش رویدادها و مناسبت‌های رسمی ایران (خورشیدی و قمری) و برچسب `(تقریبی)` برای سال‌های پس از جدول رسمی.
  - کپی سریع تاریخ در حافظه با دکمه کپی یا `Ctrl+C` و نمای جزئیات/تبدیل تاریخ سه‌گانه (شمسی ↔ میلادی ↔ قمری) با قابلیت کلیک برای کپی هر ردیف.

- **فوق‌العاده سبک و کم‌مصرف (Ultra Lightweight)**:
  - حجم فایل اجرایی تنها **~250 کیلوبایت**!
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
