use rustling::*;
use values::dimension::*;
use values::dimension::Precision::*;
use values::helpers;
use regex::Regex;
use moment::{Weekday, Grain, PeriodComp};

pub fn rule_time(b: &mut RuleSetBuilder<Dimension>) -> RustlingResult<()> {
    b.rule_2("intersect",
        time_check!(|time: &TimeValue| !time.latent),
        time_check!(|time: &TimeValue| !time.latent),
        |a, b| a.value().intersect(b.value())
    );
    b.rule_3("intersect by \",\"",
        time_check!(|time: &TimeValue| !time.latent),
        b.reg(r#","#)?,
        time_check!(|time: &TimeValue| !time.latent),
        |a, _, b| a.value().intersect(b.value())
    );
    b.rule_2("<date>에",
        time_check!(),
        b.reg(r#"에"#)?,
        |time, _| Ok(time.value().clone())
    );
    b.rule_2("<named-day>에", // on Wed, March 23
        time_check!(form!(Form::DayOfWeek{..})),
        b.reg(r#"에"#)?,
        |time, _| Ok(time.value().clone())
    );
    b.rule_2("<named-month>에", //in September
        time_check!(form!(Form::Month(_))),
        b.reg(r#"에"#)?,
        |time, _| Ok(time.value().clone())
    );
    b.rule_1("day-of-week",
        b.reg(r#"(월|화|수|목|금|토|일)(요일|욜)"#)?,
        |text_match| {
            let dow = match text_match.group(1).as_ref() {
                "월" => Weekday::Mon, 
                "화" => Weekday::Tue, 
                "수" => Weekday::Wed,
                "목" => Weekday::Thu, 
                "금" => Weekday::Fri, 
                "토" => Weekday::Sat, 
                "일" => Weekday::Sun,
                _ => panic!("Unknow match {:?}", text_match),
            };
            helpers::day_of_week(dow)
        }
    );
    b.rule_2("month",
        integer_check!(1, 12),
        b.reg(r#"월"#)?,
        |integer, _| helpers::month(integer.value().value as u32)
    );
    b.rule_2("day",
        integer_check!(1, 31),
        b.reg(r#"일"#)?,
        |integer, _| helpers::day_of_month(integer.value().value as u32)
    );
    b.rule_1("day with korean number - 십일..삼십일일",
        b.reg(r#"([이|삼]?십[일|이|삼|사|오|육|칠|팔|구]?)일"#)?,
        |text_match| {
            fn map_number(s: char) -> i64 {
                match s {
                    '일' => 1, 
                    '이' => 2, 
                    '삼' => 3, 
                    '사' => 4, 
                    '오' => 5, 
                    '육' => 6, 
                    '칠' => 7, 
                    '팔' => 8, 
                    '구' => 9, 
                    '십' => 1,
                    _ => 0,
                }
            }

            fn get_number(s: &str) -> RuleResult<i64> {
                let regex = Regex::new(r#"(.*십)?(.*)?"#)?;
                let groups = helpers::find_regex_group(&regex, s)?
                    .into_iter()
                    .nth(0)
                    .ok_or_else(|| format!("Regex {:?} has no match for {:?}", regex, s))?
                    .groups;
                let number = 10 * groups.get(1).and_then(|g| *g)
                                          .and_then(|g| g.chars().nth(0))
                                          .map(|g| map_number(g))
                                          .unwrap_or(0)
                            + groups.get(2).and_then(|g| *g)
                                          .and_then(|g| g.chars().nth(0))
                                          .map(|g| map_number(g))
                                          .unwrap_or(0);
                Ok(number)
            }
            let number = get_number(text_match.group(1));
            helpers::day_of_month(number? as u32)
        }
    );
    b.rule_1("day with korean number - 일일..구일",
        b.reg(r#"([일|이|삼|사|오|육|칠|팔|구])일"#)?,
        |text_match| {
            let dom = match text_match.group(1).as_ref() {
                "일" => 1, 
                "이" => 2, 
                "삼" => 3, 
                "사" => 4, 
                "오" => 5, 
                "육" => 6, 
                "칠" => 7, 
                "팔" => 8, 
                "구" => 9,
                _ => panic!("Unknown match {:?}", text_match)
            };
            helpers::day_of_month(dom)
        }
    );
    b.rule_1("New Year's Day",
        b.reg(r#"신정|설날"#)?,
        |_| helpers::month_day(1, 1)
    );
    b.rule_1("Independence Movement Day",
        b.reg(r#"삼일절"#)?,
        |_| helpers::month_day(3, 1)
    );
    b.rule_1("Children's Day",
        b.reg(r#"어린이날"#)?,
        |_| helpers::month_day(5, 5)
    );
    b.rule_1("Memorial Day",
        b.reg(r#"현충일"#)?,
        |_| helpers::month_day(6, 6)
    );
    b.rule_1("Constitution Day",
        b.reg(r#"제헌절"#)?,
        |_| helpers::month_day(6, 17)
    );
    b.rule_1("Liberation Day",
        b.reg(r#"광복절"#)?,
        |_| helpers::month_day(8, 15)
    );
    b.rule_1("National Foundation Day",
        b.reg(r#"개천절"#)?,
        |_| helpers::month_day(10, 3)
    );
    b.rule_1("Hangul Day",
        b.reg(r#"한글날"#)?,
        |_| helpers::month_day(10, 9)
    );
    b.rule_1("christmas eve",
        b.reg(r#"(크리스마스)?이브"#)?,
        |_| helpers::month_day(12, 24)
    );
    b.rule_1("christmas",
        b.reg(r#"크리스마스"#)?,
        |_| helpers::month_day(12, 25)
    );
    b.rule_2("absorption of , after named day",
        time_check!(form!(Form::DayOfWeek{..})),
        b.reg(r#","#)?,
        |dow, _| Ok(dow.value().clone())
    );
    b.rule_1("now",
        b.reg(r#"방금|지금|방금|막"#)?,
        |_| helpers::cycle_nth(Grain::Second, 0)
    );
    b.rule_1("today",
        b.reg(r#"오늘|당일|금일"#)?,
        |_| helpers::cycle_nth(Grain::Day, 0)
    );
    b.rule_1("tomorrow",
        b.reg(r#"내일|명일|낼"#)?,
        |_| helpers::cycle_nth(Grain::Day, 1)
    );
    b.rule_1("yesterday",
        b.reg(r#"어제"#)?,
        |_| helpers::cycle_nth(Grain::Day, -1)
    );
    b.rule_2("end of <time>",
        time_check!(),
        b.reg(r#"말"#)?,
        |time, _| time.value().the_nth(1)
    );
    b.rule_2("this <day-of-week>",
        b.reg(r#"이번주?|금주"#)?,
        time_check!(form!(Form::DayOfWeek{..})),
        |_, time| time.value().the_nth(0)
    );
    b.rule_2("this <time>",
        b.reg(r#"이번"#)?,
        time_check!(),
        |_, time| time.value().the_nth(0)
    );
    b.rule_2("next <time>",
        b.reg(r#"다음|오는"#)?,
        time_check!(|time: &TimeValue| !time.latent),
        |_, time| time.value().the_nth(1)
    );
    b.rule_2("last <time>",
        b.reg(r#"전|저번|지난"#)?,
        time_check!(),
        |_, time| time.value().the_nth(-1)
    );
    b.rule_3("<time> 마지막 <day-of-week>",
        time_check!(),
        b.reg(r#"마지막"#)?,
        time_check!(form!(Form::DayOfWeek{..})),
        |a, _, b| b.value().last_of(a.value())
    );
    b.rule_3("<time> 마지막 <cycle>",
        time_check!(),
        b.reg(r#"마지막"#)?,
        cycle_check!(),
        |time, _, cycle| cycle.value().last_of(time.value())

    );
    b.rule_3("<time> nth <time> - 3월 첫째 화요일",
        time_check!(),
        ordinal_check!(),
        time_check!(),
        |a, ordinal, b| a.value()
                .intersect(b.value())?
                .the_nth(ordinal.value().value - 1)
    );
    b.rule_1("year",
        integer_check!(1500, 2100),
        |integer| helpers::year(integer.value().value as i32)
    );
    b.rule_1("year (latent)",
        integer_check!(-1000, 999),
        |integer| Ok(helpers::year(integer.value().value as i32)?.latent())

    );
    b.rule_1("year (latent)",
        integer_check!(2101, 2300),
        |integer| Ok(helpers::year(integer.value().value as i32)?.latent())

    );
    b.rule_2("year",
        integer_check!(1),
        b.reg(r#"년"#)?,
        |integer, _| helpers::year(integer.value().value as i32)
    );
    b.rule_1("time-of-day (latent)",
        integer_check!(0, 23),
        |integer| Ok(helpers::hour(integer.value().value as u32, true)?.latent())
    );
    b.rule_2("time-of-day",
        integer_check!(0, 24),
        b.reg(r#"시"#)?,
        |integer, _| helpers::hour(integer.value().value as u32, true)
    );
    b.rule_2("<time-of-day>에",
        time_check!(form!(Form::TimeOfDay(_))),
        b.reg(r#"에"#)?,
        |time, _| Ok(time.value().clone().not_latent())
    );
    b.rule_2("<time-of-day> 정각",
        time_check!(form!(Form::TimeOfDay(_))),
        b.reg(r#"정각"#)?,
        |time, _| Ok(time.value().clone().not_latent())
    );
    b.rule_1("hh:mm",
        b.reg(r#"(?i)((?:[01]?\d)|(?:2[0-3]))[:.]([0-5]\d)"#)?,
        |text_match| helpers::hour_minute(
            text_match.group(1).parse()?,
            text_match.group(2).parse()?,
            true)
    );
    b.rule_1("hh:mm:ss",
        b.reg(r#"(?i)((?:[01]?\d)|(?:2[0-3]))[:.]([0-5]\d)[:.]([0-5]\d)"#)?,
        |text_match| helpers::hour_minute_second(
            text_match.group(1).parse()?,
            text_match.group(2).parse()?,
            text_match.group(3).parse()?,
            true
        )
    );
    b.rule_2("hhmm (military) am|pm",
        b.reg(r#"((?:1[012]|0?\d))([0-5]\d)"#)?,
        b.reg(r#"([ap])\.?m?\.?"#)?,
        |a, b| {
            let day_period = if b.group(1) == "a" {
                helpers::hour(0, false)?.span_to(&helpers::hour(12, false)?, false)?
            } else {
                helpers::hour(12, false)?.span_to(&helpers::hour(0, false)?, false)?
            };
            Ok(helpers::hour_minute(
                                a.group(1).parse()?,
                                a.group(2).parse()?, 
                                true)?.intersect(&day_period)?.form(Form::TimeOfDay(None)))
        }
    );
    // TODO: check if this rule makes sense
    b.rule_2("<time-of-day> am|pm",
        time_check!(form!(Form::TimeOfDay(_))),
        b.reg(r#"(?:in the )?([ap])(?:\s|\.)?m?\.?"#)?,
        |a, text_match| {
            let day_period = if text_match.group(1) == "a" {
                helpers::hour(0, false)?.span_to(&helpers::hour(12, false)?, false)?
            } else {
                helpers::hour(12, false)?.span_to(&helpers::hour(0, false)?, false)?
            };
            Ok(a.value().intersect(&day_period)?.form(Form::TimeOfDay(None)))
        }
    );

    b.rule_2("am|pm <time-of-day>",
        b.reg(r#"오전|아침|오후|저녁"#)?,
        time_check!(form!(Form::TimeOfDay(_))),
        |text_match, time| {
            let day_period = if text_match.group(0) == "오전" || text_match.group(0) == "아침" {
                helpers::hour(0, false)?.span_to(&helpers::hour(12, false)?, false)?
            } else {
                helpers::hour(12, false)?.span_to(&helpers::hour(0, false)?, false)?
            };
            Ok(time.value().intersect(&day_period)?.form(Form::TimeOfDay(None)))
        }
    );
    b.rule_1("noon",
        b.reg(r#"정오"#)?,
        |_| helpers::hour(12, false)
    );
    b.rule_1("midnight|EOD|end of day",
        b.reg(r#"자정"#)?,
        |_| helpers::hour(0, false)
    );
    b.rule_1("half (relative minutes)",
        b.reg(r#"반"#)?,
        |_| Ok(RelativeMinuteValue(30))
    );
    b.rule_2("number (as relative minutes)",
        integer_check!(1, 59),
        b.reg(r#"분"#)?,
        |integer, _| Ok(RelativeMinuteValue(integer.value().value as i32))
    );
    b.rule_2("<hour-of-day> <integer> (as relative minutes)",
        time_check!(form!(Form::TimeOfDay(Some(_)))),
        relative_minute_check!(),
        |tod, relative_minutes| helpers::hour_relative_minute(
            tod.value().form_time_of_day()?.full_hour,
            relative_minutes.value().0,
            true
        )
    );
    b.rule_3("<integer> (hour-of-day) relative minutes 전",
        time_check!(form!(Form::TimeOfDay(Some(_)))),
        relative_minute_check!(),
        b.reg(r#"전"#)?,
        |tod, relative_minutes, _| helpers::hour_relative_minute(
            tod.value().form_time_of_day()?.full_hour,
            -1 * relative_minutes.value().0,
            true
        )
    );
    b.rule_2("seconds",
        integer_check!(1, 59),
        b.reg(r#"초"#)?,
        |integer, _| helpers::second(integer.value().value as u32)
    );
    b.rule_1("mm/dd/yyyy", //TODO wrong rule name it should be "yyyy/mm/dd"
        b.reg(r#"(\d{2,4})[-/](0?[1-9]|1[0-2])[/-](3[01]|[12]\d|0?[1-9])"#)?,
        |text_match| helpers::ymd(
            text_match.group(1).parse()?,
            text_match.group(2).parse()?,
            text_match.group(3).parse()?
        )
    );
    b.rule_1("yyyy-mm-dd",
        b.reg(r#"(\d{2,4})-(0?[1-9]|1[0-2])-(3[01]|[12]\d|0?[1-9])"#)?,
        |text_match| helpers::ymd(
            text_match.group(1).parse()?,
            text_match.group(2).parse()?,
            text_match.group(3).parse()?
        )

    );
    b.rule_1("mm/dd",
        b.reg(r#"(0?[1-9]|1[0-2])/(3[01]|[12]\d|0?[1-9])"#)?,
        |text_match| helpers::month_day(text_match.group(1).parse()?, text_match.group(2).parse()?)

    );
    b.rule_1("morning",
        b.reg(r#"아침"#)?,
        |_| Ok(helpers::hour(4, false)?
                .span_to(&helpers::hour(12, false)?, false)?
                .latent()
                .form(Form::PartOfDay))

    );
    b.rule_1("afternoon",
        b.reg(r#"오후"#)?,
        |_| Ok(helpers::hour(12, false)?
                .span_to(&helpers::hour(19, false)?, false)?
                .latent()
                .form(Form::PartOfDay))

    );
    b.rule_1("evening|night",
        b.reg(r#"저녁|밤"#)?,
        |_| Ok(helpers::hour(18, false)?
                .span_to(&helpers::hour(0, false)?, false)?
                .latent()
                .form(Form::PartOfDay))
    );
    b.rule_1("lunch",
        b.reg(r#"점심"#)?,
        |_| Ok(helpers::hour(12, false)?
                .span_to(&helpers::hour(14, false)?, false)?
                .latent()
                .form(Form::PartOfDay))
    );
    b.rule_2("in|during the <part-of-day>",
        time_check!(form!(Form::PartOfDay)),
        b.reg(r#"에|동안"#)?,
        |time, _| Ok(time.value().clone().not_latent())

    );
    // b.rule_2("after <part-of-day>",
    //     time_check!(form!(Form::PartOfDay)),
    //     b.reg(r#"지나서|후에"#)?,
    //     |time, _|
    //         helpers::cycle_nth(Grain::Day, 0)?
    //             intersect( & helpers

    //                 )

    // );

    b.rule_2("<time> <part-of-day>",
        time_check!(),
        time_check!(form!(Form::PartOfDay)),
        |time, pod| pod.value().intersect(time.value())
    );

    b.rule_1("week-end",
        b.reg(r#"주말"#)?,
        |_| {
            let friday = helpers::day_of_week(Weekday::Fri)?
                                .intersect(&helpers::hour(18, false)?)?;
            let monday = helpers::day_of_week(Weekday::Mon)?
                                .intersect(&helpers::hour(0, false)?)?;
            friday.span_to(&monday, false)
        }
    );
    b.rule_1("season",
        b.reg(r#"여름"#)?,
        |_| helpers::month_day(6, 21)?.span_to(&helpers::month_day(9, 23)?, false)
    );
    b.rule_1("season",
        b.reg(r#"가을"#)?,
        |_| helpers::month_day(9, 23)?.span_to(&helpers::month_day(12, 21)?, false)
    );
    b.rule_1("season",
        b.reg(r#"겨울"#)?,
        |_| helpers::month_day(12, 21)?.span_to(&helpers::month_day(3, 20)?, false)
    );
    b.rule_1("season",
        b.reg(r#"봄"#)?,
        |_| helpers::month_day(3, 20)?.span_to(&helpers::month_day(6, 21)?, false)
    );
    b.rule_2("<time-of-day> approximately",
        time_check!(form!(Form::TimeOfDay(_))),
        b.reg(r#"정도|쯤"#)?,
        |time, _| Ok(time.value().clone().not_latent().precision(Precision::Approximate))
    );
    b.rule_2("about <time-of-day>",
        b.reg(r#"대충|약"#)?,
        time_check!(form!(Form::TimeOfDay(_))),
        |_, time| Ok(time.value().clone().not_latent().precision(Precision::Approximate))
    );
    b.rule_2("exactly <time-of-day>",
        time_check!(form!(Form::TimeOfDay(_))),
        b.reg(r#"정각"#)?,
        |time, _| Ok(time.value().clone().not_latent().precision(Precision::Approximate))
    );
    b.rule_3("<datetime> - <datetime> (interval)",
        time_check!(|time: &TimeValue| !time.latent),
        b.reg(r#"\-|\~|부터"#)?,
        time_check!(|time: &TimeValue| !time.latent),
        |a, _, b| a.value().span_to(b.value(), true)
    );
    b.rule_3("<time-of-day> - <time-of-day> (interval)",
        time_check!(|time: &TimeValue| if let Form::TimeOfDay(_) = time.form { !time.latent } else { false }),
        b.reg(r#"\-|\~|부터"#)?,
        time_check!(form!(Form::TimeOfDay(_))),
        |a, _, b| a.value().span_to(b.value(), true)
    );
    b.rule_2("within <duration>",
        duration_check!(),
        b.reg(r#"이내에?"#)?,
        |duration, _| helpers::cycle_nth(Grain::Second, 0)?
            .span_to(&duration.value().in_present()?, false)
    );
    b.rule_2("by <time> - 까지",
        time_check!(),
        b.reg(r#"까지"#)?,
        |time, _| helpers::cycle_nth(Grain::Second, 0)?.span_to(time.value(), false)
    );
    b.rule_2("<time-of-day>이전",
        time_check!(),
        b.reg(r#"이?전"#)?,
        |time, _| Ok(time.value().clone().direction(Some(Direction::Before)))

    );
    b.rule_2("after <time-of-day>",
        time_check!(),
        b.reg(r#"지나서|이?후에?"#)?,
        |time, _| Ok(time.value().clone().direction(Some(Direction::After)))
    );
    b.rule_2("since <time-of-day>",
        time_check!(),
        b.reg(r#"이래로"#)?,
        |time, _| Ok(time.value().the_nth(-1)?.direction(Some(Direction::After)))
    );
    Ok(())
}

pub fn rules_duration(b: &mut RuleSetBuilder<Dimension>) -> RustlingResult<()> {
    b.rule_1("second (unit-of-duration)",
        b.reg(r#"초"#)?,
        |_| Ok(UnitOfDurationValue::new(Grain::Second))
    );
    b.rule_1("minute (unit-of-duration)",
        b.reg(r#"분"#)?,
        |_| Ok(UnitOfDurationValue::new(Grain::Minute))
    );
    b.rule_1("hour (unit-of-duration)",
        b.reg(r#"시간?"#)?,
        |_| Ok(UnitOfDurationValue::new(Grain::Hour))
    );
    b.rule_1("day (unit-of-duration)",
        b.reg(r#"날|일(?:간|동안)?"#)?,
        |_| Ok(UnitOfDurationValue::new(Grain::Day))
    );
    b.rule_1("week (unit-of-duration)",
        b.reg(r#"주일?"#)?,
        |_| Ok(UnitOfDurationValue::new(Grain::Week))
    );
    b.rule_1("month (unit-of-duration)",
        b.reg(r#"(?:달)(?:간|동안)?"#)?,
        |_| Ok(UnitOfDurationValue::new(Grain::Month))
    );
    // TODO check if the quarter duration is needed
    b.rule_1("year (unit-of-duration)",
        b.reg(r#"해|연간|년(?:간|동안)?"#)?,
        |_| Ok(UnitOfDurationValue::new(Grain::Year))
    );
    // TODO check that a cycle is ncessary for this rule and not a unit of duration (hour)
    b.rule_2("half an hour",
        cycle_check!(|cycle: &CycleValue| cycle.grain == Grain::Hour),
        b.reg(r#"반"#)?,
        |_, _| Ok(DurationValue::new(PeriodComp::minutes(30).into()))
    );
    b.rule_1("a day - 하루",
        b.reg(r#"하루"#)?,
        |_| Ok(DurationValue::new(PeriodComp::days(1).into()))
    );
    b.rule_2("<integer> <unit-of-duration>",
        integer_check!(0),
        unit_of_duration_check!(),
        |integer, uod| Ok(DurationValue::new(PeriodComp::new(uod.value().grain, integer.value().value).into()))
    );
    b.rule_2("number.number hours",
        b.reg(r#"(\d+)\.(\d+)"#)?,
        b.reg(r#"시간"#)?,
        |text_match, _| {
            let decimal_hour = helpers::decimal_hour_in_minute(text_match.group(1), text_match.group(2))?;
            Ok(DurationValue::new(PeriodComp::new(Grain::Minute, decimal_hour).into()))
        }
    );
    b.rule_2("<integer> and an half hours",
        integer_check!(0),
        b.reg(r#"시간반"#)?,
        |integer, _| Ok(DurationValue::new(PeriodComp::new(Grain::Minute, integer.value().value * 60 + 30).into()))
    );
    b.rule_2("in <duration>",
        duration_check!(),
        b.reg(r#"(?:안|내)에?"#)?,
        |duration, _| duration.value().in_present()
    );
    b.rule_2("after <duration>",
        duration_check!(),
        b.reg(r#"이?후"#)?,
        |duration, _| Ok(duration
                            .value()
                            .in_present()?
                            .direction(Some(Direction::After)))
    );
    b.rule_2("<duration> from now",
        b.reg(r#"지금부터"#)?,
        duration_check!(),
        |_, duration| duration.value().in_present()
    );
    b.rule_2("<duration> ago",
        duration_check!(),
        b.reg(r#"이?전"#)?,
        |duration, _| duration.value().ago()
    );
    b.rule_2("about <duration>",
        b.reg(r#"대충|약"#)?,
        duration_check!(),
        |_, duration| Ok(duration.value().clone().precision(Precision::Approximate))
    );
    b.rule_2("exactly <duration>",
        b.reg(r#"정확히"#)?,
        duration_check!(),
        |_, duration| Ok(duration.value().clone().precision(Precision::Exact))
    );
    Ok(())
}

pub fn rules_cycle(b: &mut RuleSetBuilder<Dimension>) -> RustlingResult<()> {
    b.rule_1("second (cycle)",
        b.reg(r#"초"#)?,
        |_| CycleValue::new(Grain::Second)
    );
    b.rule_1("minute (cycle)",
        b.reg(r#"분"#)?,
        |_| CycleValue::new(Grain::Minute)
    );
    b.rule_1("hour (cycle)",
        b.reg(r#"시간?"#)?,
        |_| CycleValue::new(Grain::Hour)
    );
    b.rule_1("day (cycle)",
        b.reg(r#"날|일(?:간|동안)?"#)?,
        |_| CycleValue::new(Grain::Day)
    );
    b.rule_1("week (cycle)",
        b.reg(r#"주(?:간|동안)?"#)?,
        |_| CycleValue::new(Grain::Week)
    );
    b.rule_1("month (cycle)",
        b.reg(r#"(?:달)(?:간|동안)?"#)?,
        |_| CycleValue::new(Grain::Month)
    );
    b.rule_1("quarter (cycle)",
        b.reg(r#"분기(?:간|동안)?"#)?,
        |_| CycleValue::new(Grain::Quarter)
    );
    b.rule_1("year (cycle)",
        b.reg(r#"해|연간|년(?:간|동안)?"#)?,
        |_| CycleValue::new(Grain::Year)
    );
    b.rule_2("this <cycle>",
        b.reg(r#"이번|금|올"#)?,
        cycle_check!(),
        |_, a| helpers::cycle_nth(a.value().grain, 0)
    );
    b.rule_2("last <cycle>",
        b.reg(r#"지난|작|전|저번"#)?,
        cycle_check!(),
        |_, a| helpers::cycle_nth(a.value().grain, -1)
    );
    b.rule_2("next <cycle>",
        b.reg(r#"다음|오는|차|내"#)?,
        cycle_check!(),
        |_, a| helpers::cycle_nth(a.value().grain, 1)
    );
    b.rule_3("<time> 마지막 <cycle>",
        time_check!(),
        b.reg(r#"다음|오는|차|내"#)?,
        cycle_check!(),
        |time, _, cycle| cycle.value().last_of(time.value())
    );
    b.rule_3("<time> <ordinal> <cycle>",
        time_check!(),
        ordinal_check!(),
        cycle_check!(),
        |time, ordinal, cycle| helpers::cycle_nth_after_not_immediate(cycle.value().grain, ordinal.value().value - 1, time.value())
    );
    b.rule_1("the day after tomorrow - 내일모래",
        b.reg(r#"(?:내일)?모래"#)?,
        |_| helpers::cycle_nth_after(Grain::Day, 1, &helpers::cycle_nth(Grain::Day, 1)?)
    );
    b.rule_1("the day before yesterday - 엊그제",
        b.reg(r#"엊?그[제|재]"#)?,
        |_| helpers::cycle_nth_after(Grain::Day, -1, &helpers::cycle_nth(Grain::Day, -1)?)
    );
    b.rule_3("last n <cycle>",
        b.reg(r#"지난"#)?,
        integer_check!(1, 9999),
        cycle_check!(),
        |_, integer, cycle| helpers::cycle_n_not_immediate(cycle.value().grain, -1 * integer.value().value)
    );
    b.rule_3("next n <cycle>",
        b.reg(r#"다음"#)?,
        integer_check!(1, 9999),
        cycle_check!(),
        |_, integer, cycle| helpers::cycle_n_not_immediate(cycle.value().grain, integer.value().value)
    );
    b.rule_2("<1..4> quarter",
        integer_check!(1, 4),
        cycle_check!(|cycle: &CycleValue| cycle.grain == Grain::Quarter),
        |integer, _| helpers::cycle_nth_after(Grain::Quarter, integer.value().value - 1, &helpers::cycle_nth(Grain::Year, 0)?)
    );
    b.rule_3("<year> <1..4>quarter",
        time_check!(),
        integer_check!(1, 4),
        cycle_check!(|cycle: &CycleValue| cycle.grain == Grain::Quarter),
        |time, integer, _| helpers::cycle_nth_after(Grain::Quarter, integer.value().value - 1, time.value())
    );
    Ok(())
}


pub fn rules_numbers(b: &mut RuleSetBuilder<Dimension>) -> RustlingResult<()> {
    b.rule_1("integer (numeric)",
        b.reg(r#"(\d{1,18})"#)?,
        |text_match| {
            let value: i64 = text_match.group(1).parse()?;
            IntegerValue::new(value)
    });
    b.rule_1("integer with thousands separator ,",
        b.reg(r#"(\d{1,3}(,\d\d\d){1,5})"#)?,
        |text_match| {
            let reformatted_string = text_match.group(1).replace(",", "");
            let value: i64 = reformatted_string.parse()?;
            IntegerValue::new(value)
        }
    );
    b.rule_1("integer 0",
        b.reg(r#"영|공|빵"#)?,
        |_| IntegerValue::new(0)
    );

    b.rule_1("half - 반",
        b.reg(r#"반"#)?,
        |_| FloatValue::new(0.5)
    );
    b.rule_1("few 몇",
        b.reg(r#"몇"#)?,
        |_|  Ok(IntegerValue {
                value: 3,
                precision: Approximate,
                .. IntegerValue::default()
            })
    );
    b.rule_1("integer - TYPE 1",
        b.reg(r#"[일|이|삼|사|오|육|칠|팔|구|십|백|천|만|억|조]+"#)?,
        |text_match| {
            fn map_number(s: char) -> i64 {
                match s {
                    '일' => 1, 
                    '이' => 2, 
                    '삼' => 3, 
                    '사' => 4, 
                    '오' => 5, 
                    '육' => 6, 
                    '칠' => 7, 
                    '팔' => 8, 
                    '구' => 9, 
                    '천' => 1, 
                    '백' => 1, 
                    '십' => 1,
                    _ => 0,
                }
            }

            fn get_number(s: &str) -> RuleResult<i64> {
                let regex = Regex::new(r#"(.*천)?(.*백)?(.*십)?(.*)?"#)?;
                let groups = helpers::find_regex_group(&regex, s)?
                    .into_iter()
                    .nth(0)
                    .ok_or_else(|| format!("Regex {:?} has no match for {:?}", regex, s))?
                    .groups;
                let number = 1000 * groups.get(1).and_then(|g| *g)
                                          .and_then(|g| g.chars().nth(0))
                                          .map(|g| map_number(g))
                                          .unwrap_or(0)
                            + 100 * groups.get(2).and_then(|g| *g)
                                          .and_then(|g| g.chars().nth(0))
                                          .map(|g| map_number(g))
                                          .unwrap_or(0)
                            + 10 * groups.get(3).and_then(|g| *g)
                                          .and_then(|g| g.chars().nth(0))
                                          .map(|g| map_number(g))
                                          .unwrap_or(0)
                            + groups.get(4).and_then(|g| *g)
                                          .and_then(|g| g.chars().nth(0))
                                          .map(|g| map_number(g))
                                          .unwrap_or(0);
                Ok(number)
            }

            let regex = Regex::new(r#"(.*조)?(.*억)?(.*만)?(.*)?"#)?;
            let groups = helpers::find_regex_group(&regex, text_match.group(0))?
                    .into_iter()
                    .nth(0)
                    .ok_or_else(|| format!("Regex {:?} has no match for {:?}", regex, text_match.group(0)))?
                    .groups;

            let value = 1000000000000 * groups.get(1).and_then(|g| *g)
                                              .map(|g| get_number(g))
                                              .unwrap_or(Ok(0))?
                        + 100000000 * groups.get(2).and_then(|g| *g)
                                            .map(|g| get_number(g))
                                            .unwrap_or(Ok(0))?
                        + 10000 * groups.get(3).and_then(|g| *g)
                                        .map(|g| if g == "만" { Ok(1) } else { get_number(g)})
                                        .unwrap_or(Ok(0))?
                        + groups.get(4).and_then(|g| *g)
                                            .map(|g| get_number(g))
                                            .unwrap_or(Ok(0))?;

            IntegerValue::new(value)
        }
    );
    b.rule_1("integer (1..10) - TYPE 2",
        b.reg(r#"(하나|둘|셋|넷|다섯|여섯|일곱|여덟|아홉)"#)?,
        |text_match| {
            let value = match text_match.group(1).as_ref() {
                     "하나" => 1,
                     "둘" => 2,
                     "셋" => 3,
                     "넷" => 4,
                     "다섯" => 5,
                     "여섯" => 6,
                     "일곱" => 7,
                     "여덟" => 8,
                     "아홉" => 9,
                     _ => panic!("Unknow match"),
                 };
            IntegerValue::new(value)
        }
    );
    b.rule_1("integer (1..4) - for ordinals",
        b.reg(r#"(한|첫|두|세|네)"#)?,
        |text_match| {
            let value = match text_match.group(1).as_ref() {
                "한" => 1,
                "첫" => 1, 
                "두" => 2,
                "세" => 3,
                "네" => 4,
                _ => panic!("Unknow match"),
            };
            IntegerValue::new(value)
        }
    );

    b.rule_1("integer (20..90) - TYPE 2 and ordinals",
        b.reg(r#"(열|스물|서른|마흔|쉰|예순|일흔|여든|아흔)"#)?,
        |text_match| {
            let value = match text_match.group(1).as_ref() {
                "열"   => 10, 
                "스물" => 20, 
                "서른" => 30, 
                "마흔" => 40, 
                "쉰"   => 50,
                "예순" => 60, 
                "일흔" => 70, 
                "여든" => 80, 
                "아흔" => 90,
                _ => panic!("Unknow match"),
            };
            IntegerValue::new(value)
        }
    );
    // previous name "integer (21..99) - TYPE 2"
    b.rule_2("integer (11..99) - TYPE 2",
        integer_check!(10, 90, |integer: &IntegerValue| integer.value % 10 == 0),
        integer_check!(1, 9),
        |a, b| IntegerValue::new(a.value().value + b.value().value)
    );

    b.rule_1("decimal number",
        b.reg(r#"(\d*\.\d+)"#)?,
        |text_match| FloatValue::new(text_match.group(1).parse()?)
    );

    b.rule_2("number dot number - 삼점사",
        number_check!(|number: &NumberValue| !number.prefixed()),
        b.reg(r#"(점|쩜)([일|이|삼|사|오|육|칠|팔|구|영]+)"#)?,
        |a, text_match| {
            fn number_mapping(c: char) -> Option<char> {
                match c {
                    '일' => Some('1'), 
                    '이' => Some('2'), 
                    '삼' => Some('3'),
                    '사' => Some('4'), 
                    '오' => Some('5'),
                    '육' => Some('6'),
                    '칠' => Some('7'),
                    '팔' => Some('8'),
                    '구' => Some('9'), 
                    '영' => Some('0'),
                     _   => None,
                }
            }
            let number_string = format!("0.{}", 
                                    text_match.group(2).chars()
                                    .filter_map(number_mapping)
                                    .collect::<String>());
            FloatValue::new(a.value().value() + number_string.parse::<f32>()?)
        }
    );

    b.rule_1("decimal with thousands separator",
        b.reg(r#"(\d+(,\d\d\d)+\.\d+)"#)?,
        |text_match| FloatValue::new(text_match.group(1).replace(",", "").parse()?)
    );
    b.rule_2("numbers prefix with -, 마이너스, or 마이나스",
        b.reg(r#"-|마이너스\s?|마이나스\s?"#)?,
        number_check!(|number: &NumberValue| !number.prefixed()),
        |_, a| -> RuleResult<NumberValue> {
            Ok(match a.value().clone() {
                   NumberValue::Integer(integer) => {
                       IntegerValue {
                               value: integer.value * -1,
                               prefixed: true,
                               ..integer
                           }
                           .into()
                   }
                   NumberValue::Float(float) => {
                       FloatValue {
                               value: float.value * -1.0,
                               prefixed: true,
                               ..float
                           }
                           .into()
                   }
            })
        }
    );
    b.rule_2("ordinals (첫번째)",
        integer_check!(),
        b.reg(r#"번째|째|째번"#)?,
        |a, _| Ok(OrdinalValue { value: a.value().value })
    );
    b.rule_3("fraction",
        number_check!(|number: &NumberValue| !number.prefixed()),
        b.reg(r#"분(의|에)"#)?,
        number_check!(|number: &NumberValue| !number.suffixed()),
        |a, _, b| FloatValue::new(b.value().value() / a.value().value())
    );
    b.rule_3("fraction",
        number_check!(|number: &NumberValue| !number.prefixed()),
        b.reg(r#"/"#)?,
        number_check!(|number: &NumberValue| !number.suffixed()),
        |a, _, b| FloatValue::new(a.value().value() / b.value().value())
    );
    Ok(())
}