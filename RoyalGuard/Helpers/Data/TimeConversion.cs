using System;

namespace RoyalGuard.Helpers.Data
{
    public class TimeConversion
    {
        /*
         * Takes time and converts it into ms
         * Uses a simple switch/return statement
         */
        public long ConvertTime(long givenTime, string parameter)
        {
            long newTime = 0;

            switch(parameter)
            {
                case "s":
                    newTime = givenTime * 1000;
                    return newTime;
                case "m":
                    newTime = givenTime * 60000;
                    return newTime;
                case "h":
                    newTime = givenTime * 3600000;
                    return newTime;
                case "d":
                    newTime = givenTime * 86400000;
                    return newTime;
                case "w":
                    newTime = givenTime * 604800000;
                    return newTime;
            }

            return newTime;
        }
    }
}
