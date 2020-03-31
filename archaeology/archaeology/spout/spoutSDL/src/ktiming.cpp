// Linux kernel-assisted timing library -- provides high-precision time
// measurements for the execution time of your algorithms.
//
// It also provides timing functionality on other platforms such as Cygwin and
// Darwin for portability, but the meaning of the time reported may vary.  For
// example, on Linux we try to measure CPU time used.  On Linux, timing sleep(1)
// will report close to zero time elapsed, while on Darwin and Cygwin it will
// report the wall time, which is about 1 second.

// We need _POSIX_C_SOURCES to pick up 'struct timespec' and clock_gettime.
#define _POSIX_C_SOURCE 200112L

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#ifndef __APPLE__
#include <time.h>
#else
#include "CoreServices/CoreServices.h"
#include "mach/mach.h"
#include "mach/mach_time.h"
#endif

#include "./ktiming.h"


// ********************************* Macros *********************************

#ifdef __CYGWIN__
// Which clock to get the time from.
#define KTIMING_CLOCK_ID CLOCK_REALTIME
#else
// Which clock to get the time from.
#define KTIMING_CLOCK_ID CLOCK_PROCESS_CPUTIME_ID
#endif


// ******************************* Functions ********************************
#ifdef __APPLE__
static mach_timebase_info_data_t s_timebase_info;
uint64_t ConvAbsoluteToNanoseconds(uint64_t now){
    if (s_timebase_info.denom == 0) {
        (void) mach_timebase_info(&s_timebase_info);
    }
    
    return now / s_timebase_info.denom * s_timebase_info.numer;
}
#endif

clockmark_t ktiming_getmark() {
#ifdef __APPLE__
  const uint64_t now = mach_absolute_time();
  return ConvAbsoluteToNanoseconds(now);
#else
  struct timespec now;
  uint64_t now_nanoseconds;

  int stat = clock_gettime(KTIMING_CLOCK_ID, &now);
  if (stat != 0) {
    // Whoops, we couldn't get hold of the clock.  If we're on a
    // platform that supports it, we try again with
    // CLOCK_MONOTONIC.
#ifndef __CYGWIN__
    stat = clock_gettime(CLOCK_MONOTONIC , &now);
#endif
    if (stat != 0) {
      // Wow, we /still/ couldn't get hold of the clock.
      // Better give up; without a clock, we can't give back
      // meaningful statistics.
      perror("ktiming_getmark()");
      exit(-1);
    }
  }

  now_nanoseconds = now.tv_nsec;
  now_nanoseconds += ((uint64_t)now.tv_sec) * 1000 * 1000 * 1000;
  return now_nanoseconds;
#endif
}

uint64_t ktiming_diff_usec(const clockmark_t *const start,
                           const clockmark_t *const end) {
  return *end - *start;
}

float ktiming_diff_sec(const clockmark_t *const start,
                       const clockmark_t *const end) {
  return (float)ktiming_diff_usec(start, end) / 1000000000.0f;
}

