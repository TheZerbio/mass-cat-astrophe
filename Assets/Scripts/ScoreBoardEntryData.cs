using System;

// Shape of the JSON returned by GET /game/{id}/statistics
// JsonUtility only fills plain [Serializable] classes with public fields (no records, no properties),
// and the field names must match the JSON keys exactly.
public class ScoreBoardEntryData
{
        [Serializable]
        public class HttpReturn
        {
                public SingleReturn[] players;
        }

        [Serializable]
        public class SingleReturn
        {
                public int idle_time;
                public bool is_master;
                public string name;
                public double score;
                public double[] similarities_agg;
                public string status;
        }
}
