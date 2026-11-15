public class ScoreBoardEntryData
{
        public record HttpReturn
        {
                public SingleReturn[] players;
        }

        public record SingleReturn
        { 
                public int idle_time;
                public bool is_master;
                public string name;
                public double score;
                public double[] similarities_agg;
                public string status;
        }
}